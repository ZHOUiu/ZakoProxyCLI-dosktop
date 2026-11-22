# ZakoAI Proxy 0.14.2 · Android

APK：`ZakoAIProxy-0.14.2-android.apk`（本地调试签名，可直接安装）

- 包名 `io.zakoai.proxy`，版本 0.14.2（versionCode 1402），minSdk 26（Android 8.0），targetSdk 35
- ABI：`arm64-v8a` / `armeabi-v7a` / `x86_64`
- 权限：网络、前台服务、通知（后台保活）
- 控制台资源随 APK 内置，离线可用

## 安装与使用

1. 允许「安装未知来源应用」，安装 APK。
2. 打开 ZakoAI Proxy：首页显示运行状态、监听端口、数据目录与最近日志。
3. 服务监听 `127.0.0.1:1990`；就绪后可以：
   - 「浏览器打开控制台」→ 交给系统浏览器；
   - 「应用内打开」→ APK 内置 WebView 直接进控制台。
4. 控制台首次访问需创建管理员（密码 ≥ 12 位），然后按「分组 → 账号 → 密钥」配置。
5. 「账号」页的预设选择器支持粘贴导入（JSON / env 块 / curl 命令自动解析）。
6. 前台通知常驻避免后台被冻结；「后台运行设置」可跳转电池优化白名单。

## 说明

- 服务端是 Rust 编译的 `libzakoai.so`，随 APK 的 `lib/<abi>/` 打包，通过 JNI 在 App 进程内的
  独立线程运行（Android 10+ 禁止 exec 应用私有目录里的可执行文件，所以不能像桌面那样跑 exe）。
- 数据保存在 App 私有目录 `files/data`，卸载即清除。
- 覆盖安装请使用同一签名（本包为本地调试签名）。
- 「检查更新」指向 <https://github.com/ZHOUiu/ZakoProxyCLI-dosktop> 的 releases。

## 从源码重建

```powershell
# 1) 三 ABI 交叉编译（Windows 主机，NDK 路径必须是纯 ASCII）
$env:ANDROID_NDK_HOME = 'D:\zkbuild\v0140\ndkroot\android-ndk-r27c'
$env:ANDROID_API      = '28'
$env:JNILIBS_OUT      = 'D:\zkbuild\v0142\android\app\src\main\jniLibs'
cd D:\zkbuild\v0142\src; powershell -File .\build-android.ps1

# 2) 打 APK
cd D:\zkbuild\v0142\android
$env:ANDROID_HOME     = 'D:\zkbuild\v0140\sdkroot'
$env:ANDROID_SDK_ROOT = 'D:\zkbuild\v0140\sdkroot'
$env:GRADLE_USER_HOME = 'D:\zkbuild\v0140\gradlehome'
$env:ZAKO_WEB_DIST    = 'D:\zkbuild\v0142\src\web\dist'
D:\zkbuild\v0140\gradleroot\gradle-8.9\bin\gradle.bat assembleRelease
# 产物 app\build\outputs\apk\release\app-release.apk
```
