package io.zakoai.proxy;

import android.app.Activity;
import android.content.Intent;
import android.graphics.Color;
import android.os.Bundle;
import android.view.Gravity;
import android.view.ViewGroup;
import android.webkit.ConsoleMessage;
import android.webkit.WebChromeClient;
import android.webkit.WebResourceError;
import android.webkit.WebResourceRequest;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.widget.FrameLayout;
import android.widget.TextView;

/**
 * In-app console over the embedded relay.
 *
 * Only ever opened by the "应用内打开" button: the launcher never navigates here by itself.
 */
public class ConsoleActivity extends Activity {
    private FrameLayout container;
    private WebView web;
    private TextView hint;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        container = new FrameLayout(this);
        container.setBackgroundColor(0xFFFFFAF3);
        hint = new TextView(this);
        hint.setText("正在连接本地服务…");
        hint.setTextColor(0xFF7A6A65);
        hint.setTextSize(15);
        hint.setGravity(Gravity.CENTER);
        container.addView(hint, new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        setContentView(container);
        loadWhenReady();
    }

    private void loadWhenReady() {
        new Thread(() -> {
            long deadline = System.currentTimeMillis() + 30_000;
            while (System.currentTimeMillis() < deadline) {
                if (ProxyRunner.probe(800)) {
                    runOnUiThread(this::attachWebView);
                    return;
                }
                try {
                    Thread.sleep(350);
                } catch (InterruptedException e) {
                    return;
                }
            }
            runOnUiThread(() -> hint.setText("本地服务未响应。\n返回上一页查看启动日志。"));
        }, "zakoai-console-wait").start();
    }

    private void attachWebView() {
        if (web != null || isFinishing()) {
            return;
        }
        web = new WebView(this);
        WebSettings settings = web.getSettings();
        settings.setJavaScriptEnabled(true);
        settings.setDomStorageEnabled(true);
        settings.setDatabaseEnabled(true);
        settings.setUseWideViewPort(true);
        settings.setLoadWithOverviewMode(true);
        settings.setSupportZoom(false);
        settings.setTextZoom(100);
        settings.setCacheMode(WebSettings.LOAD_NO_CACHE);
        settings.setMixedContentMode(WebSettings.MIXED_CONTENT_COMPATIBILITY_MODE);
        settings.setUserAgentString(settings.getUserAgentString() + " ZakoAIProxy/" + BuildInfo.VERSION);
        web.setBackgroundColor(Color.parseColor("#FFFAF3"));
        web.setWebViewClient(new WebViewClient() {
            @Override
            public boolean shouldOverrideUrlLoading(WebView view, String url) {
                return url == null || !url.startsWith("http://127.0.0.1:" + ProxyProcess.PORT);
            }

            @Override
            public void onPageFinished(WebView view, String url) {
                view.evaluateJavascript(
                        "(function(){var r=document.getElementById('root');"
                                + "return JSON.stringify({ready:document.readyState,"
                                + "children:r?r.children.length:-1});})()",
                        value -> ProxyRunner.log("page state: " + value));
            }

            @Override
            public void onReceivedError(WebView view, WebResourceRequest request, WebResourceError error) {
                ProxyRunner.log("page error " + request.getUrl() + " -> " + error.getErrorCode()
                        + " " + error.getDescription());
            }
        });
        web.setWebChromeClient(new WebChromeClient() {
            @Override
            public boolean onConsoleMessage(ConsoleMessage message) {
                ProxyRunner.log("console " + message.messageLevel() + " @"
                        + message.sourceId() + ":" + message.lineNumber()
                        + " " + message.message());
                return true;
            }
        });
        container.addView(web, new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        container.removeView(hint);
        web.loadUrl(BuildInfo.ORIGIN + "/");
    }

    @Override
    public void onBackPressed() {
        if (web != null && web.canGoBack()) {
            web.goBack();
            return;
        }
        startActivity(new Intent(this, MainActivity.class));
        finish();
    }

    @Override
    protected void onDestroy() {
        if (web != null) {
            container.removeView(web);
            web.destroy();
            web = null;
        }
        super.onDestroy();
    }
}
