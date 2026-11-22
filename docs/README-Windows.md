# ZakoAI Proxy 0.14.2 · Windows 10/11 x64

两种形态，任选其一：

| 文件 | 说明 |
| --- | --- |
| `ZakoAI-Proxy-0.14.2-Windows-x64.exe` | **单文件**：内嵌服务端 DLL 与控制台资源，首次运行自动展开 |
| `ZakoAI-Proxy-0.14.2-Windows-x64.zip` | **目录版**：解压即用，`start.bat` 或 `ZakoAI.Desktop.exe` 启动 |

## 单文件版用法

1. 把 `ZakoAI-Proxy-0.14.2-Windows-x64.exe` 放到任意可写目录（如 `D:\Apps\ZakoAI\`）。
2. 双击运行。首次启动会从 exe 尾部展开这几项：

   ```
   zakoai_proxy.dll   ← 服务端（Rust cdylib，由启动器进程内加载）
   web\               ← 控制台静态资源
   zako.ico
   .payload           ← 展开标记，用于跳过重复解包
   ```

   若程序目录不可写（例如放在 `C:\Program Files`），自动改用
   `%LOCALAPPDATA%\ZakoAIProxy\` 作为工作目录，不会再弹权限错误。

3. 状态栏出现「运行中 http://127.0.0.1:1990」后，点「打开控制台」——
   交给**系统默认浏览器**打开，保留你自己的浏览器配置与密码管理器。
4. 控制台首次访问需创建管理员（密码 ≥ 12 位），随后按「分组 → 账号 → 密钥」配置。
5. 「账号」页新增**预设选择器**：选好厂商预设后可直接粘贴 `{"api_key":…}`、
   `OPENAI_API_KEY=…`、`export ANTHROPIC_BASE_URL=…` 或整条 curl 命令，
   服务端会解析出 endpoint 与凭证并预填表单。

## 目录版用法

解压 zip 后直接运行 `start.bat` 或 `ZakoAI.Desktop.exe`，其余流程同上。
目录版不需要解包步骤，`web\`、`zakoai_proxy.dll` 与 `zakoai-proxy.exe` 已在包内
（`zakoai-proxy.exe` 是没有启动器时的回退入口）。

## 目录结构（首次运行后）

```
ZakoAI-Proxy-0.14.2-Windows-x64.exe   ← 程序本体（含内嵌载荷）
zakoai_proxy.dll                      ← 首次运行自动展开（进程内宿主）
web\                                  ← 控制台资源（展开得到）
data\                                 ← SQLite 数据库、master.key
logs\desktop.log                      ← 启动器与服务端日志
zako.ico
```

## 关于「检查更新」

控制台的更新源指向 <https://github.com/ZHOUiu/ZakoProxyCLI-dosktop>：

- 版本面板调用 `api.github.com/repos/ZHOUiu/ZakoProxyCLI-dosktop/releases/latest`；
- 只接受该仓库（以及上游 `mizaawa/ZakoProxyCLI`，容错保留）的正式稳定 release 页；
- Docker 部署路径仍按官方镜像名构建，桌面/Android 包不涉及。

## 常见问题

| 现象 | 处理 |
| --- | --- |
| 杀软报毒 / 直接删掉 `zakoai-proxy.exe` | 这是本地代理服务端（自建 AI 中转），部分安全软件（如抖音安全组件、企业 EDR）会拦。单文件版只带 DLL，不含独立 exe；目录版被删时改用 `ZakoAI.Desktop.exe` 启动，或把整个目录加入白名单 |
| 端口 1990 被占用 | 关掉占用进程，或先在控制台「设置」里改绑定地址 |
| 想让局域网访问 | 控制台设置里把绑定地址改成 `0.0.0.0:1990`，并在防火墙放行 |
| 控制台空白 | 先看 `logs\desktop.log`；确认 `web\` 与 exe 同目录。仍空白时点「打开控制台」用系统浏览器访问 |
| 双击无反应 | 检查是否被杀软阻止；用 `ZakoAI.Desktop.exe` 启动可看到窗口与日志 |

## 从源码重建

```powershell
cd D:\zkbuild\v0142\src
cargo build --release                                  # zakoai-proxy.exe + zakoai_proxy.dll
cd web; npm ci; npm run build; cd ..                   # web\dist
powershell -File ..\scripts\pack-win.ps1               # 目录 zip + 单文件 exe
```

关键点：`Cargo.toml` 里 `[lib] crate-type = ["rlib","cdylib"]`，`src/launcher.rs`
导出 `zako_launch`；启动器用 `LoadLibrary("zakoai_proxy.dll")` +
`GetProcAddress("zako_launch")` 在自身进程内起服务（64 MB 栈线程）。
