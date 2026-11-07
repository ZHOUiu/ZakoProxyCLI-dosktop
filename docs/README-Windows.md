# ZakoAI Proxy 0.13.3 · Windows 10/11 x64

两种形态，任选其一：

| 文件 | 说明 |
| --- | --- |
| `ZakoAI-Proxy-0.13.3-Windows-x64.exe` | **单文件**：内嵌服务端与控制台资源，首次运行自动展开到同目录 |
| `ZakoAI-Proxy-0.13.3-Windows-x64.zip` | **目录版**：解压即用，`start.bat` 或 `ZakoAI.Desktop.exe` 启动 |

## 单文件版用法

1. 把 `ZakoAI-Proxy-0.13.3-Windows-x64.exe` 放到任意可写目录（如 `D:\Apps\ZakoAI\`）。
2. 双击运行。首次启动会从 exe 尾部展开 5 个组成部分：

   ```
   zakoai-proxy.exe   ← 服务端（Rust）
   web\               ← 控制台静态资源
   zako.ico
   .payload           ← 展开标记，用于跳过重复解包
   ```

   若程序目录不可写（例如放在 `C:\Program Files`），自动改用
   `%LOCALAPPDATA%\ZakoAIProxy\` 作为工作目录，不会再弹权限错误。

3. 状态栏出现「运行中 http://127.0.0.1:1990」后，点「打开控制台」——
   交给**系统默认浏览器**打开，保留你自己的浏览器配置与密码管理器。
4. 控制台首次访问需创建管理员（密码 ≥ 12 位），随后按「分组 → 账号 → 密钥」配置。

## 目录版用法

解压 zip 后直接运行 `start.bat` 或 `ZakoAI.Desktop.exe`，其余流程同上。
目录版不需要解包步骤，`web\` 与 `zakoai-proxy.exe` 已在包内。

## 目录结构（首次运行后）

```
ZakoAI-Proxy-0.13.3-Windows-x64.exe   ← 程序本体（含内嵌载荷）
zakoai-proxy.exe                      ← 首次运行自动展开
web\                                  ← 控制台资源（展开得到）
data\                                 ← SQLite 数据库、master.key
logs\desktop.log                      ← 启动器与服务端日志
zako.ico
```

## 常见问题

| 现象 | 处理 |
| --- | --- |
| 杀软报毒 / 直接删掉 `zakoai-proxy.exe` | 这是本地代理服务端（自建 AI 中转），部分安全软件（如抖音安全组件、企业 EDR）会拦。把整个程序目录加入白名单，或换一台干净的机器运行 |
| 端口 1990 被占用 | 关掉占用进程，或先在控制台「设置」里改绑定地址 |
| 想让局域网访问 | 控制台设置里把绑定地址改成 `0.0.0.0:1990`，并在防火墙放行 |
| 控制台空白 | 先看 `logs\desktop.log`；确认 `web\` 与 exe 同目录。仍空白时点「打开控制台」用系统浏览器访问 |
| 双击无反应 | 检查是否被杀软阻止；用 `ZakoAI.Desktop.exe` 启动可看到窗口与日志 |

## 从源码重建

```powershell
cd src\ZakoProxyCLI-0.13.3
cargo build --release                                  # zakoai-proxy.exe
cd web; npm ci; npm run build; cd ..                   # web\dist
csc /target:winexe /out:ZakoAI.Desktop.exe ..\windows\ZakoAI.Desktop.cs `
    /r:System.dll /r:System.Drawing.dll /r:System.Windows.Forms.dll `
    /r:System.IO.Compression.dll /r:System.IO.Compression.FileSystem.dll
powershell -File ..\scripts\pack-win.ps1               # zip + 单文件 exe
```
