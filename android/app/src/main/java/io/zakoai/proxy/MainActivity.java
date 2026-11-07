package io.zakoai.proxy;

import android.annotation.SuppressLint;
import android.app.Activity;
import android.content.ActivityNotFoundException;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.os.PowerManager;
import android.provider.Settings;
import android.view.Gravity;
import android.view.View;
import android.view.ViewGroup;
import android.view.Window;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;
import android.widget.Toast;

import java.io.File;

/** Launcher view: shows the embedded relay state and opens the console. */
public class MainActivity extends Activity {
    private TextView status;
    private TextView detail;
    private TextView log;
    private Button openButton;
    private Button inAppButton;
    private Button restartButton;
    private final Handler handler = new Handler(Looper.getMainLooper());


    private final Runnable refresh = new Runnable() {
        @Override
        public void run() {
            render();
            handler.postDelayed(this, 1500);
        }
    };

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            if (checkSelfPermission("android.permission.POST_NOTIFICATIONS") != PackageManager.PERMISSION_GRANTED) {
                requestPermissions(new String[]{"android.permission.POST_NOTIFICATIONS"}, 7);
            }
        }
        ProxyService.start(this);
        ProxyRunner.boot(this);
        setContentView(buildView());

        handler.post(refresh);
    }

    @Override
    protected void onResume() {
        super.onResume();
        render();
    }

    @Override
    protected void onDestroy() {
        handler.removeCallbacks(refresh);
        super.onDestroy();
    }

    private View buildView() {
        LinearLayout root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setBackgroundColor(0xFFFFFAF3);
        int pad = dp(24);
        root.setPadding(pad, pad, pad, pad);

        TextView title = new TextView(this);
        title.setText("ZakoAI Proxy");
        title.setTextSize(28);
        title.setTextColor(0xFF5A4B46);
        root.addView(title);

        TextView sub = new TextView(this);
        sub.setText("本地 AI API 中转 · 控制台 http://127.0.0.1:" + ProxyProcess.PORT);
        sub.setTextSize(13);
        sub.setTextColor(0xFF9A8A85);
        LinearLayout.LayoutParams subLp = new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT);
        subLp.topMargin = dp(4);
        root.addView(sub, subLp);

        status = new TextView(this);
        status.setTextSize(16);
        status.setTextColor(0xFFE991AC);
        LinearLayout.LayoutParams statusLp = new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT);
        statusLp.topMargin = dp(20);
        root.addView(status, statusLp);

        detail = new TextView(this);
        detail.setTextSize(12);
        detail.setTextColor(0xFF7A6A65);
        LinearLayout.LayoutParams detailLp = new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT);
        detailLp.topMargin = dp(6);
        root.addView(detail, detailLp);

        LinearLayout buttons = new LinearLayout(this);
        buttons.setOrientation(LinearLayout.HORIZONTAL);
        LinearLayout.LayoutParams buttonsLp = new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT);
        buttonsLp.topMargin = dp(20);
        root.addView(buttons, buttonsLp);

        openButton = new Button(this);
        openButton.setText("浏览器打开控制台");
        openButton.setEnabled(false);
        openButton.setOnClickListener(v -> openConsole());
        buttons.addView(openButton, new LinearLayout.LayoutParams(0,
                ViewGroup.LayoutParams.WRAP_CONTENT, 1f));

        inAppButton = new Button(this);
        inAppButton.setText("应用内打开");
        inAppButton.setEnabled(false);
        inAppButton.setOnClickListener(v -> openInAppConsole());
        buttons.addView(inAppButton, new LinearLayout.LayoutParams(0,
                ViewGroup.LayoutParams.WRAP_CONTENT, 1f));

        restartButton = new Button(this);
        restartButton.setText("检查状态");
        restartButton.setOnClickListener(v -> {
            // Probing /health needs a socket, so it must not run on the UI thread.
            new Thread(() -> {
                boolean alive = ProxyRunner.probe(1500);
                if (!alive) {
                    Intent intent = new Intent(this, ProxyService.class);
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                        startForegroundService(intent);
                    } else {
                        startService(intent);
                    }
                }
                runOnUiThread(this::render);
            }, "zakoai-check").start();
        });
        buttons.addView(restartButton, new LinearLayout.LayoutParams(0,
                ViewGroup.LayoutParams.WRAP_CONTENT, 1f));

        Button battery = new Button(this);
        battery.setText("后台运行设置");
        battery.setOnClickListener(v -> requestBatteryExemption());
        LinearLayout.LayoutParams batteryLp = new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT);
        batteryLp.topMargin = dp(8);
        root.addView(battery, batteryLp);

        ScrollView scroll = new ScrollView(this);
        log = new TextView(this);
        log.setTextSize(10);
        log.setTextColor(0xFF8A7A75);
        log.setTypeface(android.graphics.Typeface.MONOSPACE);
        scroll.addView(log);
        LinearLayout.LayoutParams logLp = new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f);
        logLp.topMargin = dp(16);
        root.addView(scroll, logLp);
        return root;
    }

    private void render() {
        String state = ProxyRunner.state();
        status.setText(state);
        boolean up = state.startsWith("运行中");
        openButton.setEnabled(up);
        inAppButton.setEnabled(up);
        String lan = NetworkInfo.lanAddress(this);
        File files = getFilesDir();
        StringBuilder info = new StringBuilder();
        info.append(BuildInfo.describe()).append('\n');
        info.append("端口 ").append(ProxyProcess.PORT).append("（仅本机 127.0.0.1）\n");
        if (lan != null) {
            info.append("同一局域网可访问 ").append(lan).append(':').append(ProxyProcess.PORT)
                    .append("（需改 BIND_ADDR 为 0.0.0.0）\n");
        }
        info.append("数据目录 ").append(new File(files, "data").getAbsolutePath()).append('\n');
        info.append("日志 ").append(new File(files, "logs/server.log").getAbsolutePath());
        detail.setText(info.toString());
        try {
            File serverLog = new File(files, "logs/server.log");
            String tail = NetworkInfo.readTail(serverLog, 40);
            if (tail.isEmpty()) {
                java.util.List<String> lines = ProxyRunner.lines();
                StringBuilder sb = new StringBuilder();
                int from = Math.max(0, lines.size() - 40);
                for (int i = from; i < lines.size(); i++) {
                    sb.append(lines.get(i)).append('\n');
                }
                tail = sb.toString();
            }
            log.setText(tail);
        } catch (Exception e) {
            log.setText("日志不可用：" + e);
        }
        // Nothing opens by itself: the console is one explicit tap away.
        if (up) {
            openButton.setEnabled(true);
            inAppButton.setEnabled(true);
        }
    }

    /** Hands the console URL to whatever browser the user has set as default. */
    private void openConsole() {
        Intent view = new Intent(Intent.ACTION_VIEW, Uri.parse(BuildInfo.ORIGIN + "/"));
        view.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
        try {
            startActivity(view);
            ProxyRunner.log("opened console in default browser: " + BuildInfo.ORIGIN + "/");
        } catch (ActivityNotFoundException missing) {
            ProxyRunner.log("no browser available: " + missing);
            Toast.makeText(this, "未找到可用的浏览器，请手动访问 " + BuildInfo.ORIGIN,
                    Toast.LENGTH_LONG).show();
        }
    }

    /**
     * Opens the console inside the app's own WebView; only ever on this explicit tap.
     *
     * No readiness probe here: HttpURLConnection on the UI thread throws
     * NetworkOnMainThreadException on API 26+, and ConsoleActivity already waits for
     * /health on a background thread before it loads the page.
     */
    private void openInAppConsole() {
        try {
            startActivity(new Intent(this, ConsoleActivity.class));
            ProxyRunner.log("opened console in app");
        } catch (Exception error) {
            ProxyRunner.log("in-app console failed: " + error);
            Toast.makeText(this, "应用内控制台不可用：" + error.getMessage(),
                    Toast.LENGTH_LONG).show();
        }
    }

    @SuppressLint("BatteryLife")
    private void requestBatteryExemption() {
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                PowerManager power = getSystemService(PowerManager.class);
                if (power != null && power.isIgnoringBatteryOptimizations(getPackageName())) {
                    startActivity(new Intent(Settings.ACTION_IGNORE_BATTERY_OPTIMIZATION_SETTINGS));
                    return;
                }
            }
            startActivity(new Intent(Settings.ACTION_IGNORE_BATTERY_OPTIMIZATION_SETTINGS));
        } catch (ActivityNotFoundException e) {
            startActivity(new Intent(Settings.ACTION_SETTINGS));
        }
    }

    private int dp(int value) {
        return Math.round(value * getResources().getDisplayMetrics().density);
    }
}
