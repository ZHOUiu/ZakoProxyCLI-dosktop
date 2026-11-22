# Cross-compiles zakoai-proxy as libzakoai.so for the Android console APK (Windows host).
#
# The app hosts the relay in-process (Android 10+ forbids exec'ing a binary from app
# storage), so the whole server is linked as a shared library driven by the JNI entry
# point in src/embedded.rs.
#
# Requires:
#   ANDROID_NDK_HOME  -> an ASCII-only NDK path (NDK clang cannot write outputs when its
#                        own path contains non-ASCII characters)
#   the three android rust targets already added via `rustup target add`

$ErrorActionPreference = 'Stop'

$ndk = $env:ANDROID_NDK_HOME
if (-not $ndk) { throw 'set ANDROID_NDK_HOME to the NDK root' }
if (-not (Test-Path $ndk)) { throw "ANDROID_NDK_HOME not found: $ndk" }
if ($ndk -match '[^\x00-\x7F]') { throw "NDK path contains non-ASCII characters: $ndk" }

$api = if ($env:ANDROID_API) { $env:ANDROID_API } else { '28' }
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$out = if ($env:JNILIBS_OUT) { $env:JNILIBS_OUT } else { Join-Path $root 'target\jniLibs' }
$toolchain = Join-Path $ndk 'toolchains\llvm\prebuilt\windows-x86_64\bin'
$llvmAr = Join-Path $toolchain 'llvm-ar.exe'

$targets = @(
    @{ triple = 'aarch64-linux-android';    prefix = 'aarch64-linux-android';    abi = 'arm64-v8a' },
    @{ triple = 'armv7-linux-androideabi';  prefix = 'armv7a-linux-androideabi'; abi = 'armeabi-v7a' },
    @{ triple = 'x86_64-linux-android';     prefix = 'x86_64-linux-android';     abi = 'x86_64' }
)

foreach ($t in $targets) {
    $linker = Join-Path $toolchain "$($t.prefix)$api-clang.cmd"
    if (-not (Test-Path $linker)) { throw "linker not found: $linker" }
    $upper = $t.triple.ToUpperInvariant().Replace('-', '_')
    Write-Host "==> $($t.triple) -> $($t.abi)"

    $env:CARGO_TARGET_DIR = Join-Path $root 'target\android'
    Set-Item -Path "env:CARGO_TARGET_${upper}_LINKER" -Value $linker
    Set-Item -Path "env:CC_$($t.triple)" -Value $linker
    Set-Item -Path "env:AR_$($t.triple)" -Value $llvmAr
    $env:RUSTFLAGS = "-C linker=$linker"

    Push-Location $root
    try {
        cargo build --release --locked --target $t.triple --lib
        if ($LASTEXITCODE -ne 0) { throw "cargo build failed for $($t.triple)" }
    }
    finally {
        Pop-Location
    }

    $built = Join-Path $root "target\android\$($t.triple)\release\libzakoai_proxy.so"
    if (-not (Test-Path $built)) { throw "missing $built" }
    $dest = Join-Path $out $t.abi
    New-Item -ItemType Directory -Force -Path $dest | Out-Null
    Copy-Item $built (Join-Path $dest 'libzakoai.so') -Force
    "    {0:N0} bytes -> {1}" -f (Get-Item (Join-Path $dest 'libzakoai.so')).Length, $dest | Write-Host
}

Write-Host "done: $out"
