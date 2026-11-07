# ZakoAI Proxy · Windows 10/11 与 Android 打包仓

上游：[mizaawa/ZakoProxyCLI](https://github.com/mizaawa/ZakoProxyCLI)
本仓基线：**v0.13.3**（同步上游 0.13.3 的「聊天思考过程」改动后重新打包）

## 成品

本仓**只放源码**，二进制产物不进版本库（见 `.gitignore`）。按下面「从源码重建」跑完
`scripts/pack-win.ps1` 与 `gradle assembleRelease` 即可得到：

| 产物 | 说明 | 参考大小 |
| --- | --- | --- |
| `ZakoAI-Proxy-0.13.3-Windows-x64.exe` | Windows 单文件版，双击即用（自动展开 DLL 与控制台资源） | 4.9 MB |
| `ZakoAI-Proxy-0.13.3-Windows-x64.zip` | Windows 目录版，解压后跑 `start.bat` | 28.4 MB |
| `ZakoAIProxy-0.13.3-android.apk` | Android 安装包，包名 `io.zakoai.proxy`，minSdk 26 / targetSdk 35 | 27.8 MB |
| `SHA256SUMS.txt` | 三个产物的校验值（打包脚本自动生成） | — |

## 仓库结构

```
src/ZakoProxyCLI-0.13.3/   上游 0.13.3 源码 + 本次改造（Rust / 前端）
android/                   Android 工程（AGP 8.7.3 / Gradle 8.9 / JDK 21）
windows/                   Windows 启动器源码 ZakoAI.Desktop.cs
scripts/                   打包脚本 pack-win.ps1
docs/                      两个平台的 README 与实测截图
assets/                    图标（ico / png）
```

## 与上游的差异

### 1. 便携化（沿用 0.13.2 时期改造，已随 0.13.3 重编）

- `src/ZakoProxyCLI-0.13.3/src/presets.rs`：新增 26 条服务商预设与 `AuthStyle` 枚举
- `api.rs`：`/api/presets`、`/api/accounts/parse-credential` 两个接口 + 粘贴式凭据解析
- `proxy.rs`：按账号 `auth_style` 分派 `Authorization` / `x-api-key` / `x-goog-api-key`；
  修正 `/v1beta/openai` 被二次拼 `/v1` 的 404
- `lib.rs`：静态资源补 `Content-Type` 与 `nosniff`，缺失资源返回 404（修控制台白屏）
- `main.rs`：按 exe 位置解析 `web/`、`data/`，日志落 `logs/zakoai.log`
- `embedded.rs` / `launcher.rs`：JNI 与 DLL 两个宿主入口
- `web/src/PresetPicker.tsx` 等：预设选择器与粘贴导入 UI

### 2. 本次（0.13.3）新增

- 合入上游 0.13.3 的 `Playground.tsx` / `playground.css`（聊天思考过程渲染）
- 版本号全线升到 0.13.3：`Cargo.toml`、`web/package.json`、`android/app/build.gradle.kts`（versionCode 1303）、`BuildInfo.java`
- **Windows 单文件改为「仅内嵌 DLL」**：载荷只带 `zakoai_proxy.dll` + `web/`，启动器用
  `LoadLibrary` + `zako_launch` 在**自身进程内**起服务，不再释放独立 `zakoai-proxy.exe`。
  独立 exe 会被部分终端安全软件（本机实测：抖音安全组件、火绒）判定为代理程序并**直接删除镜像**，
  进程内加载可绕开这一层；目录版仍同时提供 exe 作为回退。
- 载荷尾部用 ASCII 记录长度（`KA/OBC__AKZ?len=NNNNNNNNNN`），`.payload` 标记避免重复解包；
  程序目录不可写时自动落到 `%LOCALAPPDATA%\ZakoAIProxy`
- `build-android.sh` 重写：三个 ABI 一次编完，输出到 `jniLibs/`

## 从源码重建

```bash
# 1. Rust（Windows）
cd src/ZakoProxyCLI-0.13.3
cargo build --release            # → target/release/{zakoai-proxy.exe, zakoai_proxy.dll}

# 2. 前端
cd web && npm ci && npm run build # → web/dist

# 3. Android 原生库（三 ABI）
ANDROID_NDK_HOME=<ndk> ./build-android.sh
# 产物 target/jniLibs/<abi>/libzakoai.so → android/app/src/main/jniLibs/<abi>/

# 4. APK
cd ../../android
ANDROID_HOME=<sdk> JAVA_HOME=<jdk21> ../gradle-8.9/bin/gradle assembleRelease

# 5. Windows 打包（zip + 单文件 exe）
powershell -File ../scripts/pack-win.ps1 -BuildDir <crate> -WorkDir <work> -ReleaseDir <release>
```

> 工具链路径必须全 ASCII。NDK 的 clang 与 Gradle 的 `native-platform.dll` 在中文路径下会
> 写文件失败（`Permission denied` / `Failed to load native library`），构建时请把 crate、NDK、
> JDK、SDK 放到如 `D:\build\` 这样的纯英文目录。

## 实测记录

| 项目 | 结果 |
| --- | --- |
| Windows 目录版 | 进程内加载 DLL，`/health` = `{"status":"ok"}`，控制台可建管理员并进入仪表盘 |
| Windows 单文件版 | 首次运行展开 18 个文件，`/health` 正常，`/assets/*.js` 返回 `text/javascript` |
| Android（Android 11 / x86_64 模拟器） | 安装启动正常，首页显示「运行中 127.0.0.1:1990」，应用内 WebView 打开控制台可建管理员 |
| 0.13.3 新页面 | 「聊天」页正常渲染（思考过程 UI） |

截图见 `docs/shots/`。

## 已知限制

- 单文件版依赖本机可写目录；`C:\Program Files` 下会自动改用 `%LOCALAPPDATA%\ZakoAIProxy`
- 目录版的 `zakoai-proxy.exe` 在装了终端安全软件的机器上可能被拦（程序仍会先用进程内 DLL，
  所以不影响使用；需要独立 exe 时请把整个目录加白）
- Android 包为本地调试签名（`Android Debug`），覆盖安装需保持同一签名
