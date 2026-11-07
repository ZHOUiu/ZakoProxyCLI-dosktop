# ZakoAI Proxy 0.13.3 · Android

APK：`ZakoAIProxy-0.13.3-android.apk`（本地调试签名，可直接安装）

- 包名 `io.zakoai.proxy`，版本 0.13.3（versionCode 1303），minSdk 26（Android 8.0），targetSdk 35
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
5. 前台通知常驻避免后台被冻结；「后台运行设置」可跳转电池优化白名单。

## 说明

- 服务端是 Rust 编译的 `libzakoai.so`，随 APK 的 `lib/<abi>/` 打包，通过 JNI 在 App 进程内的
  独立线程运行（Android 10+ 禁止 exec 应用私有目录里的可执行文件，所以不能像桌面那样跑 exe）。
- 数据保存在 App 私有目录 `files/data`，卸载即清除。
- 覆盖安装请使用同一签名（本包为本地调试签名）。

## 从源码重建

```bash
cd src/ZakoProxyCLI-0.13.3
ANDROID_NDK_HOME=<sdk>/ndk/27.2.12479018 \
  cargo +nightly build --release --lib -Z build-std=std,panic_abort \
  --target aarch64-linux-android        # 同法编 armv7 / x86_64
# 产物 target/<triple>/release/libzakoai_proxy.so → android/app/src/main/jniLibs/<abi>/libzakoai.so

cd ../../android
ANDROID_HOME=<sdk> JAVA_HOME=<jdk21> ../tools/gradle-8.9/bin/gradle assembleRelease
```
