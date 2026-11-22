# ZakoAI Proxy · Windows 10/11 与 Android 打包仓

上游：[mizaawa/ZakoProxyCLI](https://github.com/mizaawa/ZakoProxyCLI)
本仓基线：**v0.14.2**（上游 0.14.2 + 本地改造三方合并后重新打包）

## 成品（`release/`）

| 文件 | 说明 | 大小 |
| --- | --- | --- |
| `ZakoAI-Proxy-0.14.2-Windows-x64.exe` | Windows 单文件版，双击即用（自动展开 DLL 与控制台资源） | 6.3 MB |
| `ZakoAI-Proxy-0.14.2-Windows-x64.zip` | Windows 目录版，解压后跑 `start.bat` 或 `ZakoAI.Desktop.exe` | 12.0 MB |
| `ZakoAIProxy-0.14.2-android.apk` | Android 安装包，包名 `io.zakoai.proxy`，minSdk 26 / targetSdk 35，三 ABI | 38.8 MB |
| `SHA256SUMS.txt` | 三个产物的校验值 | — |

## 仓库结构

```
src/ZakoProxyCLI-0.14.2/   上游 0.14.2 源码 + 本次改造（Rust / 前端）
android/                   Android 工程（AGP 8.7.3 / Gradle 8.9 / JDK 21，含三 ABI jniLibs）
windows/                   Windows 启动器源码 ZakoAI.Desktop.cs
scripts/                   打包脚本 pack-win.ps1
docs/                      两个平台的 README、改动总览与实测截图
assets/                    图标（ico / png）
release/                   三个成品与校验值
```

## 与上游的差异

### 1. 便携化改造（沿用并随 0.14.2 重编）

- `src/ZakoProxyCLI-0.14.2/src/presets.rs`：内置服务商预设目录与 `AuthStyle` 枚举
- `api.rs`：`/api/presets`、`/api/accounts/parse-credential` 两个接口 + 粘贴式凭据解析
  （JSON / env 块 / curl 命令自动解析 endpoint 与凭证）
- `proxy.rs`：按账号 `auth_style` 分派 `Authorization` / `x-api-key` / `x-goog-api-key`；
  修正 `/v1beta/openai` 等带版本路径被二次拼 `/v1` 的 404
- `lib.rs`：上游已改用 `ServeFile` + precompressed gzip（MIME 正确）；本版保留
  「security_path 为空时放行 SPA 深链」与「缺 index.html 时回 SHELL 页」
- `main.rs`：按 exe 位置解析 `web/`、`data/`，日志落 `logs/zakoai.log`
- `embedded.rs` / `launcher.rs`：JNI 与 DLL 两个宿主入口（`zako_launch` 导出）
- `web/src/PresetPicker.tsx` + `preset-picker.css`：预设选择器与粘贴导入 UI
- `profiles/zako-break/`：deepseek / glm / grok / kimi 四个家族的本地目录（目录总数 48）

### 2. 本次（0.14.2）合并说明

上游 0.14.2 变化：控制台静态服务重写为 `tower-http ServeFile`（含预压缩 gzip 支持）、
新增 `resourceCache` / `resources` 资源预热、`profiles()` 支持 `?summary=`、目录新增 2 条 profile、
前端拆包更细。本地改造按三方合并接入，冲突消解记录见 `docs/改动总览-0.14.2.md`。

### 3. 更新源已指向本仓

控制台「检查更新」与侧栏 GitHub 入口都指向 `ZHOUiu/ZakoProxyCLI-dosktop`：

- `src/update.rs`：`REPOSITORY` / `RELEASE_API` / release 页白名单（上游两个仓容错保留）
- `web/src/App.tsx`：侧栏链接

要让「检查更新」能查到版本，本仓需发布 release：tag 形如 `v0.14.2`、非 draft / 非 prerelease。

## 从源码重建

```powershell
# 1) 前端
cd src\ZakoProxyCLI-0.14.2\web
npm ci; npm run build                    # → web/dist

# 2) Windows 服务端
cd ..
cargo build --release                    # → target/release/{zakoai-proxy.exe, zakoai_proxy.dll}

# 3) Windows 成品
powershell -File ..\..\scripts\pack-win.ps1

# 4) Android 原生库（NDK 路径必须纯 ASCII）
$env:ANDROID_NDK_HOME = '<ndk>\android-ndk-r27c'
$env:ANDROID_API = '28'
$env:JNILIBS_OUT = '..\..\android\app\src\main\jniLibs'
powershell -File .\build-android.ps1

# 5) APK
cd ..\..\android
$env:ZAKO_WEB_DIST = '..\src\ZakoProxyCLI-0.14.2\web\dist'
gradle assembleRelease                  # → app/build/outputs/apk/release/app-release.apk
```

关键点：`Cargo.toml` 里 `[lib] crate-type = ["rlib","cdylib"]`，`launcher.rs` 导出 `zako_launch`，
启动器 `LoadLibrary("zakoai_proxy.dll")` 后在自身进程内起服务（64 MB 栈线程）。
Android 10+ 禁止 exec 应用私有目录的可执行文件，所以 APK 走 JNI 宿主。

## 验证记录

- `cargo test --lib`：90 passed；集成测试（除 `kimi_profiles`，本机终端安全组件拦截测试 exe）60 passed
- Windows 目录版 / 单文件版实跑：HTTP 200，`ZAKO · API Relay` 首页正常
- APK：`aapt2 dump badging` → versionCode 1402 / versionName 0.14.2 / 三 ABI；`apksigner verify` → 本地调试签名
