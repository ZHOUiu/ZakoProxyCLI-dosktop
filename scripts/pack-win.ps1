#requires -Version 5.1
<#
    Packages the Windows deliverables for ZakoAI Proxy 0.13.3.

      1. directory edition  -> <Release>\ZakoAI-Proxy-0.13.3-Windows-x64\   and .zip
      2. single file        -> <Release>\ZakoAI-Proxy-0.13.3-Windows-x64.exe
                               (launcher + appended zip payload: zakoai-proxy.exe, web\, assets\)

    The payload is a plain deflate zip appended to the launcher image; the trailing
    length is written into a 46-character UTF-16 slot that the launcher embeds
    (@'s + marker + 10 digits), so no external unpacker is needed at runtime.
#>
param(
    [Parameter(Mandatory = $true)][string]$BuildDir,      # ...\src (crate root with target\release)
    [Parameter(Mandatory = $true)][string]$WorkDir,       # ...\work
    [Parameter(Mandatory = $true)][string]$ReleaseDir,    # ...\release
    [string]$Version = '0.13.3'
)

$ErrorActionPreference = 'Stop'
$root = Join-Path $WorkDir 'scripts'
$launcherSrc = Join-Path $WorkDir 'windows\ZakoAI.Desktop.cs'
$icon = Join-Path $WorkDir 'assets\zako.ico'
$stageDir = Join-Path $WorkDir "build\ZakoAI-Proxy-$Version-Windows-x64"
$payloadStage = Join-Path $WorkDir 'build\payload'
$baseName = "ZakoAI-Proxy-$Version-Windows-x64"

function New-Directory($path) {
    if (Test-Path $path) { Remove-Item $path -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $path | Out-Null
}

function Get-Csc {
    foreach ($candidate in @(
            "$env:SystemRoot\Microsoft.NET\Framework64\v4.0.30319\csc.exe",
            "$env:SystemRoot\Microsoft.NET\Framework\v4.0.30319\csc.exe")) {
        if (Test-Path $candidate) { return $candidate }
    }
    throw 'csc.exe (.NET Framework 4.x) not found'
}

# ---------------------------------------------------------------- 1. compile shell
$exeOut = Join-Path $WorkDir 'windows\ZakoAI.Desktop.exe'
& (Get-Csc) /nologo /target:winexe /platform:anycpu /optimize+ /out:$exeOut "/win32icon:$icon" `
    $launcherSrc /r:System.dll /r:System.Core.dll /r:System.Drawing.dll /r:System.Windows.Forms.dll `
    /r:System.IO.Compression.dll /r:System.IO.Compression.FileSystem.dll
if ($LASTEXITCODE -ne 0) { throw "csc failed with $LASTEXITCODE" }
"launcher: {0:N0} bytes" -f (Get-Item $exeOut).Length | Write-Host

# ---------------------------------------------------------------- 2. stage the tree
New-Directory $stageDir
New-Item -ItemType Directory -Force -Path $payloadStage | Out-Null

$serverExe = Join-Path $BuildDir "target\release\zakoai-proxy.exe"
if (-not (Test-Path $serverExe)) { throw "missing $serverExe - run cargo build --release first" }
$webDist = Join-Path $BuildDir 'web\dist'
if (-not (Test-Path (Join-Path $webDist 'index.html'))) { throw "missing $webDist - run npm run build first" }

Copy-Item $serverExe (Join-Path $stageDir 'zakoai-proxy.exe')
Copy-Item (Join-Path $BuildDir 'target\release\zakoai_proxy.dll') (Join-Path $stageDir 'zakoai_proxy.dll')
Copy-Item $exeOut (Join-Path $stageDir 'ZakoAI.Desktop.exe')
Copy-Item $icon (Join-Path $stageDir 'zako.ico')
Copy-Item (Join-Path $BuildDir 'LICENSE') (Join-Path $stageDir 'LICENSE')
Copy-Item (Join-Path $BuildDir 'README.md') (Join-Path $stageDir 'README.upstream.md')
Copy-Item (Join-Path $WorkDir 'docs\README-Windows.md') (Join-Path $stageDir 'README-Windows.md')
Copy-Item (Join-Path $WorkDir 'docs\README-Android.md') (Join-Path $stageDir 'README-Android.md')
Copy-Item -Recurse $webDist (Join-Path $stageDir 'web')

@"
@echo off
cd /d "%~dp0"
if exist "ZakoAI.Desktop.exe" (
  start "" "ZakoAI.Desktop.exe"
) else (
  start "" "zakoai-proxy.exe"
)
"@ | Set-Content (Join-Path $stageDir 'start.bat') -Encoding ascii

# ---------------------------------------------------------------- 3. directory zip
New-Item -ItemType Directory -Force -Path $ReleaseDir | Out-Null
$zipPath = Join-Path $ReleaseDir "$baseName.zip"
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory(
    (Split-Path -Parent $stageDir),
    $zipPath,
    [System.IO.Compression.CompressionLevel]::Optimal,
    $false)
"zip: {0} ({1:N1} MB)" -f $zipPath, ((Get-Item $zipPath).Length / 1MB) | Write-Host

# ---------------------------------------------------------------- 4. payload zip
# The single-file build ships the cdylib only: the launcher hosts it in-process.
# Bundling the standalone zakoai-proxy.exe trips endpoint agents (they flag the
# relay PE and delete the whole image), so the exe stays in the directory edition.
New-Directory $payloadStage
Copy-Item (Join-Path $stageDir 'zakoai_proxy.dll') $payloadStage
Copy-Item $icon $payloadStage
Copy-Item -Recurse (Join-Path $stageDir 'web') (Join-Path $payloadStage 'web')
$payloadZip = Join-Path $WorkDir 'build\payload.zip'
if (Test-Path $payloadZip) { Remove-Item $payloadZip -Force }
[System.IO.Compression.ZipFile]::CreateFromDirectory(
    $payloadStage, $payloadZip, [System.IO.Compression.CompressionLevel]::Optimal, $false)
"payload: {0:N1} MB" -f ((Get-Item $payloadZip).Length / 1MB) | Write-Host

# ---------------------------------------------------------------- 5. build single file
$launcherBytes = [System.IO.File]::ReadAllBytes($exeOut)
$payloadBytes = [System.IO.File]::ReadAllBytes($payloadZip)
$trailer = [System.Text.Encoding]::ASCII.GetBytes(('KA/OBC__AKZ?len={0:D10}' -f $payloadBytes.Length))
$single = New-Object byte[] ($launcherBytes.Length + $payloadBytes.Length + $trailer.Length)
[Array]::Copy($launcherBytes, 0, $single, 0, $launcherBytes.Length)
[Array]::Copy($payloadBytes, 0, $single, $launcherBytes.Length, $payloadBytes.Length)
[Array]::Copy($trailer, 0, $single, $launcherBytes.Length + $payloadBytes.Length, $trailer.Length)

$singlePath = Join-Path $ReleaseDir "$baseName.exe"
[System.IO.File]::WriteAllBytes($singlePath, $single)
"single: {0} ({1:N1} MB, payload {2:N0} bytes)" -f $singlePath, ($single.Length / 1MB), $payloadBytes.Length | Write-Host

# ---------------------------------------------------------------- 6. self check
$check = [System.IO.File]::ReadAllBytes($singlePath)
$tail = [System.Text.Encoding]::ASCII.GetString($check[($check.Length - 256)..($check.Length - 1)])
$mark = 'KA/OBC__AKZ?len='
$pos = $tail.LastIndexOf($mark, [System.StringComparison]::Ordinal)
if ($pos -lt 0) { throw 'verification: trailer missing' }
$len = [int64]$tail.Substring($pos + $mark.Length, 10)
if ($len -ne $payloadBytes.Length) { throw "verification: length mismatch $len vs $($payloadBytes.Length)" }
$offset = $check.Length - $len - $trailer.Length
if ($check[$offset] -ne 0x50 -or $check[$offset + 1] -ne 0x4B) { throw 'verification: payload is not a zip' }
if ($offset -ne $launcherBytes.Length) { throw "verification: payload offset $offset != launcher length $($launcherBytes.Length)" }
"verified: trailer ok, payload $len bytes at offset $offset, zip magic PK, total $($check.Length) bytes" | Write-Host

# ---------------------------------------------------------------- 7. checksums
$manifest = Get-ChildItem $ReleaseDir -File | Where-Object { $_.Extension -in '.zip', '.exe', '.apk' } |
    ForEach-Object { '{0}  {1}  {2:N0} bytes' -f (Get-FileHash $_.FullName -Algorithm SHA256).Hash, $_.Name, $_.Length }
$manifest | Write-Host
$manifest | Set-Content (Join-Path $ReleaseDir 'SHA256SUMS.txt') -Encoding utf8
