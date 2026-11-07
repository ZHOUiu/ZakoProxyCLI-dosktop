package io.zakoai.proxy;

import android.os.Build;

/** Build facts shown on the launcher screen. */
final class BuildInfo {
    static final String VERSION = "0.13.3";
    static final String ORIGIN = "http://127.0.0.1:1990";

    private BuildInfo() {}

    static String describe() {
        String abi = Build.SUPPORTED_ABIS.length > 0 ? Build.SUPPORTED_ABIS[0] : "unknown";
        return "ZakoAI Proxy " + VERSION + " · Android " + Build.VERSION.RELEASE
                + " (API " + Build.VERSION.SDK_INT + ") · " + abi;
    }
}
