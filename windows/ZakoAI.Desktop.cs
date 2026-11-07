using System;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.IO.Compression;
using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Threading;
using System.Windows.Forms;

/// <summary>
/// Launcher window for ZakoAI Proxy.
///
/// Two distribution shapes are supported by the same binary:
///  - directory edition: zakoai-proxy.exe + web\ sit next to this exe;
///  - single file: a zip payload is appended to this exe by pack-sfx.ps1 and is
///    expanded next to the exe on first run (falls back to %LOCALAPPDATA% when
///    the program directory is read-only).
///
/// Nothing opens on start. The console is reached through the "打开控制台"
/// button, which hands the URL to the system's default browser so the operator
/// keeps their own browser profile, extensions and password manager.
/// </summary>
sealed class ShellForm : Form
{
    const string Origin = "http://127.0.0.1:1990";
    const int Port = 1990;
    internal static string Home = AppDomain.CurrentDomain.BaseDirectory;
    static readonly object LogLock = new object();

    readonly TextBox logBox = new TextBox();
    readonly Label stateLabel = new Label();
    readonly Button startButton = new Button();
    readonly Button consoleButton = new Button();
    readonly System.Windows.Forms.Timer timer = new System.Windows.Forms.Timer();
    Process server;
    volatile bool starting;

    internal ShellForm()
    {
        Text = "ZakoAI Proxy 0.13.3";
        ClientSize = new Size(940, 620);
        MinimumSize = new Size(560, 400);
        StartPosition = FormStartPosition.CenterScreen;
        BackColor = Color.FromArgb(255, 250, 243);
        Font = new Font("Microsoft YaHei UI", 9f);
        try
        {
            string icon = Path.Combine(Home, "zako.ico");
            if (File.Exists(icon))
            {
                Icon = new Icon(icon);
            }
        }
        catch { }
        Build();
        timer.Interval = 900;
        timer.Tick += (s, e) => Tick();
        timer.Start();
        Bootstrap();
    }

    internal static void Log(string line)
    {
        string text = DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss ") + line + Environment.NewLine;
        try
        {
            lock (LogLock)
            {
                string dir = Path.Combine(Home, "logs");
                Directory.CreateDirectory(dir);
                File.AppendAllText(Path.Combine(dir, "desktop.log"), text, Encoding.UTF8);
            }
        }
        catch
        {
            try
            {
                File.AppendAllText(Path.Combine(Path.GetTempPath(), "zako-desktop.log"), text, Encoding.UTF8);
            }
            catch { }
        }
    }

    static bool Health(int timeoutMs)
    {
        try
        {
            var request = (HttpWebRequest)WebRequest.Create(Origin + "/health");
            request.Timeout = timeoutMs;
            request.ReadWriteTimeout = timeoutMs;
            request.Proxy = null;
            request.KeepAlive = false;
            using (var response = (HttpWebResponse)request.GetResponse())
            {
                return (int)response.StatusCode == 200;
            }
        }
        catch
        {
            return false;
        }
    }

    static bool PortOpen()
    {
        try
        {
            using (var client = new TcpClient())
            {
                var task = client.ConnectAsync("127.0.0.1", Port);
                return task.Wait(400) && client.Connected;
            }
        }
        catch
        {
            return false;
        }
    }

    /// <summary>Expands an appended payload (if any) and then starts the server.</summary>
    void Bootstrap()
    {
        var worker = new Thread(() =>
        {
            try
            {
                string note = Payload.Ensure();
                if (note != null)
                {
                    Append(note);
                }
            }
            catch (Exception error)
            {
                Append("解包失败：" + error.Message);
                Log("payload failure: " + error);
            }
            StartServer();
        });
        worker.IsBackground = true;
        worker.Start();
    }

    void Build()
    {
        var bar = new FlowLayoutPanel
        {
            Dock = DockStyle.Top,
            Height = 52,
            Padding = new Padding(10, 10, 10, 6),
            BackColor = Color.FromArgb(255, 245, 249),
            WrapContents = false,
        };
        stateLabel.Text = "启动中…";
        stateLabel.Font = new Font("Microsoft YaHei UI", 10f, FontStyle.Bold);
        stateLabel.ForeColor = Color.FromArgb(90, 75, 70);
        stateLabel.Size = new Size(330, 30);
        stateLabel.TextAlign = ContentAlignment.MiddleLeft;
        bar.Controls.Add(stateLabel);

        startButton.Text = "启动服务";
        startButton.FlatStyle = FlatStyle.Flat;
        startButton.Size = new Size(96, 30);
        startButton.Margin = new Padding(4, 0, 0, 0);
        startButton.Click += (s, e) => StartServer();
        bar.Controls.Add(startButton);

        consoleButton.Text = "打开控制台";
        consoleButton.FlatStyle = FlatStyle.Flat;
        consoleButton.Size = new Size(112, 30);
        consoleButton.Margin = new Padding(4, 0, 0, 0);
        consoleButton.Enabled = false;
        consoleButton.Click += (s, e) => OpenConsole();
        bar.Controls.Add(consoleButton);

        var logs = new Button { Text = "打开日志", FlatStyle = FlatStyle.Flat, Size = new Size(96, 30), Margin = new Padding(4, 0, 0, 0) };
        logs.Click += (s, e) => OpenFolder(Path.Combine(Home, "logs"));
        bar.Controls.Add(logs);

        var data = new Button { Text = "数据目录", FlatStyle = FlatStyle.Flat, Size = new Size(96, 30), Margin = new Padding(4, 0, 0, 0) };
        data.Click += (s, e) => OpenFolder(Path.Combine(Home, "data"));
        bar.Controls.Add(data);

        logBox.Multiline = true;
        logBox.ReadOnly = true;
        logBox.ScrollBars = ScrollBars.Vertical;
        logBox.Dock = DockStyle.Fill;
        logBox.BorderStyle = BorderStyle.None;
        logBox.BackColor = Color.FromArgb(255, 252, 247);
        logBox.Font = new Font("Consolas", 9f);
        logBox.Text = "正在准备本地服务…" + Environment.NewLine;

        Controls.Add(logBox);
        Controls.Add(bar);
    }

    void Tick()
    {
        bool up = Health(500);
        stateLabel.Text = up ? "运行中  " + Origin : "未运行";
        stateLabel.ForeColor = up ? Color.FromArgb(60, 140, 100) : Color.FromArgb(200, 80, 100);
        startButton.Enabled = !up;
        consoleButton.Enabled = up;
    }

    void StartServer()
    {
        if (starting || Health(400))
        {
            return;
        }
        starting = true;
        try { BeginInvoke(new Action(() => startButton.Enabled = false)); } catch { }
        var worker = new Thread(() =>
        {
            try
            {
                if (PortOpen())
                {
                    Append("端口 " + Port + " 已被占用，沿用现有服务。");
                    return;
                }
                // In-process hosting first: some endpoint agents block executing a
                // standalone relay binary, while loading the same code as a DLL works.
                if (NativeServer.Start(Home))
                {
                    Append("已在进程内加载 zakoai_proxy.dll（无需独立 exe）。");
                    for (int i = 0; i < 120; i++)
                    {
                        if (Health(600))
                        {
                            Append("服务就绪，点「打开控制台」在默认浏览器中进入 " + Origin + "/");
                            return;
                        }
                        Thread.Sleep(500);
                    }
                    Append("进程内服务未响应，改用独立 exe。");
                }
                string binary = Path.Combine(Home, "zakoai-proxy.exe");
                if (!File.Exists(binary))
                {
                    Append("缺少 zakoai-proxy.exe，请把它放到程序同一目录。");
                    return;
                }
                var info = new ProcessStartInfo(binary)
                {
                    WorkingDirectory = Home,
                    UseShellExecute = false,
                    CreateNoWindow = true,
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                    StandardOutputEncoding = Encoding.UTF8,
                    StandardErrorEncoding = Encoding.UTF8,
                };
                info.EnvironmentVariables["DATA_DIR"] = Path.Combine(Home, "data");
                info.EnvironmentVariables["WEB_DIR"] = Path.Combine(Home, "web");
                info.EnvironmentVariables["BIND_ADDR"] = "127.0.0.1:" + Port;
                info.EnvironmentVariables["RUST_LOG"] = "zakoai_proxy=info";
                var process = Process.Start(info);
                server = process;
                process.OutputDataReceived += (s, e) => { if (e.Data != null) Log(e.Data); };
                process.ErrorDataReceived += (s, e) => { if (e.Data != null) Log(e.Data); };
                process.BeginOutputReadLine();
                process.BeginErrorReadLine();
                Append("已启动 zakoai-proxy.exe (pid " + process.Id + ")");
                for (int i = 0; i < 120; i++)
                {
                    if (Health(600))
                    {
                        Append("服务就绪，点「打开控制台」在默认浏览器中进入 " + Origin + "/");
                        return;
                    }
                    if (process.HasExited)
                    {
                        Append("服务进程已退出（代码 " + process.ExitCode + "）。");
                        return;
                    }
                    Thread.Sleep(500);
                }
                Append("服务未在 60 秒内响应，请查看日志。");
            }
            catch (Exception error)
            {
                Append("启动失败：" + error.Message);
                Log("start failure: " + error);
            }
            finally
            {
                starting = false;
                try
                {
                    BeginInvoke(new Action(() => startButton.Enabled = true));
                }
                catch { }
            }
        });
        worker.IsBackground = true;
        worker.Start();
    }

    /// <summary>Hands the console URL to the default browser; never launches its own.</summary>
    void OpenConsole()
    {
        if (!Health(1200))
        {
            MessageBox.Show(this, "服务尚未就绪，请稍候再试。", "ZakoAI Proxy",
                MessageBoxButtons.OK, MessageBoxIcon.Information);
            return;
        }
        try
        {
            Process.Start(new ProcessStartInfo(Origin + "/") { UseShellExecute = true });
            Append("已在默认浏览器打开 " + Origin + "/");
        }
        catch (Exception error)
        {
            Log("open console failed: " + error);
            MessageBox.Show(this, "无法调用默认浏览器：" + error.Message + Environment.NewLine +
                    "可以手动访问 " + Origin + "/", "ZakoAI Proxy",
                MessageBoxButtons.OK, MessageBoxIcon.Warning);
        }
    }

    void OpenFolder(string path)
    {
        try
        {
            Directory.CreateDirectory(path);
            Process.Start("explorer.exe", path);
        }
        catch { }
    }

    void Append(string line)
    {
        Log(line);
        if (IsDisposed)
        {
            return;
        }
        if (InvokeRequired)
        {
            try
            {
                BeginInvoke(new Action<string>((l) => logBox.AppendText(l + Environment.NewLine)), line);
            }
            catch { }
            return;
        }
        logBox.AppendText(line + Environment.NewLine);
    }

    protected override void OnFormClosed(FormClosedEventArgs e)
    {
        try
        {
            if (server != null && !server.HasExited)
            {
                server.Kill();
            }
        }
        catch { }
        base.OnFormClosed(e);
    }
}

/// <summary>
/// Appended zip payload support: the packed single-file exe ends with a deflate
/// zip of zakoai-proxy.exe plus web/, and a 26-character trailer written from
/// PackSlot so the length can be recovered without any external tool.
/// </summary>
static class Payload
{
    const string Marker = "KA/OBC__AKZ?len=";
    const string StampName = ".payload";

    static string SelfPath
    {
        get { return Process.GetCurrentProcess().MainModule.FileName; }
    }

    /// <summary>Expands the payload into Home when needed; returns a status line or null.</summary>
    internal static string Ensure()
    {
        string self = SelfPath;
        long payloadLength;
        long offset;
        if (!Locate(self, out offset, out payloadLength))
        {
            return null; // directory edition: nothing appended
        }
        string stamp = Path.Combine(ShellForm.Home, StampName);
        string want = "0.13.3:" + payloadLength;
        if (File.Exists(stamp) && File.Exists(Path.Combine(ShellForm.Home, "zakoai-proxy.exe")))
        {
            try
            {
                if (File.ReadAllText(stamp).Trim() == want)
                {
                    return "内置运行时已展开（" + payloadLength + " 字节载荷）。";
                }
            }
            catch { }
        }

        string zipPath = Path.Combine(Path.GetTempPath(), "zako-" + Guid.NewGuid().ToString("N") + ".zip");
        try
        {
            using (var source = new FileStream(self, FileMode.Open, FileAccess.Read, FileShare.ReadWrite))
            {
                source.Seek(offset, SeekOrigin.Begin);
                using (var target = new FileStream(zipPath, FileMode.Create, FileAccess.Write, FileShare.None))
                {
                    var buffer = new byte[81920];
                    long left = payloadLength;
                    while (left > 0)
                    {
                        int want2 = (int)Math.Min(buffer.Length, left);
                        int read = source.Read(buffer, 0, want2);
                        if (read <= 0)
                        {
                            throw new IOException("载荷读取提前结束");
                        }
                        target.Write(buffer, 0, read);
                        left -= read;
                    }
                }
            }

            ShellForm.Home = PickHome();
            int written = 0;
            using (var archive = ZipFile.OpenRead(zipPath))
            {
                foreach (var entry in archive.Entries)
                {
                    if (entry.FullName.EndsWith("/") || entry.FullName.EndsWith("\\"))
                    {
                        continue;
                    }
                    string destination = Path.Combine(ShellForm.Home, entry.FullName.Replace('/', Path.DirectorySeparatorChar));
                    Directory.CreateDirectory(Path.GetDirectoryName(destination));
                    if (entry.FullName.StartsWith("data/", StringComparison.OrdinalIgnoreCase) && File.Exists(destination))
                    {
                        continue; // never touch an existing database
                    }
                    try
                    {
                        entry.ExtractToFile(destination, true);
                        written++;
                    }
                    catch (IOException)
                    {
                        // A running server may hold its own image; keep the existing copy.
                    }
                }
            }
            File.WriteAllText(stamp, want, Encoding.UTF8);
            return "已展开内置运行时：" + written + " 个文件 → " + ShellForm.Home;
        }
        finally
        {
            try { File.Delete(zipPath); } catch { }
        }
    }

    /// <summary>Finds the ASCII trailer at EOF and returns where the zip starts.</summary>
    static bool Locate(string self, out long offset, out long length)
    {
        offset = 0;
        length = 0;
        var info = new FileInfo(self);
        if (info.Length < 4096)
        {
            return false;
        }
        int window = 256;
        var tail = new byte[window];
        using (var stream = new FileStream(self, FileMode.Open, FileAccess.Read, FileShare.ReadWrite))
        {
            stream.Seek(info.Length - window, SeekOrigin.Begin);
            int read = 0;
            while (read < window)
            {
                int got = stream.Read(tail, read, window - read);
                if (got <= 0)
                {
                    break;
                }
                read += got;
            }
        }
        string text = Encoding.ASCII.GetString(tail);
        int at = text.LastIndexOf(Marker, StringComparison.Ordinal);
        if (at < 0)
        {
            return false;
        }
        int digitsAt = at + Marker.Length;
        if (digitsAt + 10 > text.Length)
        {
            return false;
        }
        long parsed;
        if (!long.TryParse(text.Substring(digitsAt, 10), out parsed) || parsed <= 0 || parsed >= info.Length)
        {
            return false;
        }
        length = parsed;
        offset = info.Length - parsed - (Marker.Length + 10);
        return offset > 0;
    }

    /// <summary>Uses the program directory when writable, otherwise %LOCALAPPDATA%.</summary>
    static string PickHome()
    {
        string baseDir = AppDomain.CurrentDomain.BaseDirectory;
        if (Writable(baseDir))
        {
            return baseDir;
        }
        string fallback = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "ZakoAIProxy");
        Directory.CreateDirectory(fallback);
        return fallback;
    }

    static bool Writable(string dir)
    {
        try
        {
            string probe = Path.Combine(dir, ".zako-write-" + Guid.NewGuid().ToString("N"));
            using (var stream = new FileStream(probe, FileMode.CreateNew, FileAccess.Write, FileShare.None, 1, FileOptions.DeleteOnClose))
            {
                stream.WriteByte(0);
            }
            return true;
        }
        catch
        {
            return false;
        }
    }
}

/// <summary>
/// In-process hosting of zakoai_proxy.dll (the same Rust server, built as a
/// cdylib). zako_launch spins the axum server up on its own thread.
/// </summary>
static class NativeServer
{
    [System.Runtime.InteropServices.DllImport("kernel32", SetLastError = true, CharSet = System.Runtime.InteropServices.CharSet.Unicode)]
    static extern IntPtr LoadLibrary(string path);

    [System.Runtime.InteropServices.DllImport("kernel32", CharSet = System.Runtime.InteropServices.CharSet.Ansi, ExactSpelling = true)]
    static extern IntPtr GetProcAddress(IntPtr module, string name);

    [System.Runtime.InteropServices.UnmanagedFunctionPointer(System.Runtime.InteropServices.CallingConvention.Cdecl)]
    delegate void Launch(IntPtr instance);

    static bool loaded;

    internal static bool Start(string home)
    {
        if (loaded)
        {
            return true;
        }
        string dll = Path.Combine(home, "zakoai_proxy.dll");
        if (!File.Exists(dll))
        {
            return false;
        }
        Environment.SetEnvironmentVariable("DATA_DIR", Path.Combine(home, "data"));
        Environment.SetEnvironmentVariable("WEB_DIR", Path.Combine(home, "web"));
        Environment.SetEnvironmentVariable("BIND_ADDR", "127.0.0.1:1990");
        Environment.SetEnvironmentVariable("ZAKO_HOME", home);
        try
        {
            IntPtr module = LoadLibrary(dll);
            if (module == IntPtr.Zero)
            {
                ShellForm.Log("LoadLibrary failed: " + System.Runtime.InteropServices.Marshal.GetLastWin32Error());
                return false;
            }
            IntPtr symbol = GetProcAddress(module, "zako_launch");
            if (symbol == IntPtr.Zero)
            {
                ShellForm.Log("zako_launch not exported");
                return false;
            }
            var launch = (Launch)System.Runtime.InteropServices.Marshal.GetDelegateForFunctionPointer(symbol, typeof(Launch));
            var thread = new Thread(() => launch(IntPtr.Zero)) { IsBackground = true };
            thread.Start();
            loaded = true;
            return true;
        }
        catch (Exception error)
        {
            ShellForm.Log("native start failed: " + error);
            return false;
        }
    }
}

/// <summary>Entry point for the ZakoAI Proxy launcher.</summary>
static class Launcher
{
    [STAThread]
    static void Main()
    {
        try
        {
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);
            Application.Run(new ShellForm());
        }
        catch (Exception error)
        {
            ShellForm.Log("fatal: " + error);
            try
            {
                MessageBox.Show("ZakoAI Proxy 无法启动：" + error.Message, "ZakoAI Proxy",
                    MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
            catch { }
        }
    }
}
