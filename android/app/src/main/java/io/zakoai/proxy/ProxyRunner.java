package io.zakoai.proxy;

import android.content.Context;

import java.io.File;
import java.io.FileOutputStream;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * Boots the embedded relay in-process.
 *
 * <p>The upstream server is an async runtime plus two socket listeners; hosting it in this
 * process (instead of exec-ing a bundled binary) is what keeps it working on Android 10+
 * where W^X forbids executing files from app storage.</p>
 */
final class ProxyRunner {
    static final String BIND = "127.0.0.1:" + ProxyProcess.PORT;

    private static final AtomicBoolean STARTED = new AtomicBoolean(false);
    private static final ArrayDeque<String> LOG = new ArrayDeque<>();
    private static final int MAX_LOG = 400;
    private static volatile String state = "未启动";

    private ProxyRunner() {}

    static void boot(Context context) {
        if (!STARTED.compareAndSet(false, true)) {
            return;
        }
        state = "正在启动…";
        Thread thread = new Thread(() -> {
            File files = context.getFilesDir();
            try {
                new File(files, "tmp").mkdirs();
                new File(files, "logs").mkdirs();
                redirectStdio(files);
                Installer.syncAssets(context);
                File data = Installer.dataDir(context);
                File web = Installer.webDir(context);
                log("console dir: " + web.getAbsolutePath());
                log("data dir: " + data.getAbsolutePath());
                log("server binary: " + Installer.serverBin(context).getAbsolutePath());
                ProxyProcess.load(new File(context.getApplicationInfo().nativeLibraryDir));
                log("binding " + BIND);
                state = "运行中 " + BIND;
                ProxyProcess.serve(data.getAbsolutePath(), web.getAbsolutePath(), BIND);
                state = "已停止";
            } catch (Throwable failure) {
                state = "启动失败：" + failure.getMessage();
                log("boot failure: " + failure);
                android.util.Log.e("ZakoAI", "boot failure", failure);
            }
        }, "zakoai-server");
        thread.setDaemon(true);
        thread.start();
    }

    static String state() {
        return state;
    }

    static void log(String line) {
        android.util.Log.i("ZakoAI", line);
        synchronized (LOG) {
            LOG.addLast(line);
            while (LOG.size() > MAX_LOG) {
                LOG.removeFirst();
            }
        }
    }

    static List<String> lines() {
        synchronized (LOG) {
            return new ArrayList<>(LOG);
        }
    }

    private static void redirectStdio(File files) {
        try {
            FileOutputStream out = new FileOutputStream(new File(files, "logs/server.log"), true);
            System.setOut(new java.io.PrintStream(out, true, "UTF-8"));
            System.setErr(new java.io.PrintStream(new FileOutputStream(
                    new File(files, "logs/server.err.log"), true), true, "UTF-8"));
        } catch (Exception e) {
            log("stdio redirection unavailable: " + e);
        }
    }

    static boolean probe(int timeoutMs) {
        java.net.HttpURLConnection connection = null;
        try {
            connection = (java.net.HttpURLConnection)
                    new java.net.URL(BuildInfo.ORIGIN + "/health").openConnection();
            connection.setConnectTimeout(timeoutMs);
            connection.setReadTimeout(timeoutMs);
            connection.setUseCaches(false);
            return connection.getResponseCode() == 200;
        } catch (Exception e) {
            return false;
        } finally {
            if (connection != null) {
                connection.disconnect();
            }
        }
    }
}
