package io.zakoai.proxy;

import java.io.File;

/** Entry point into the bundled Rust relay (JNI, see src/ZakoProxyCLI-0.13.2/build-android.sh). */
final class ProxyProcess {
    static final int PORT = 1990;
    private static volatile boolean loaded;

    private ProxyProcess() {}

    /** Loads the packaged Rust server once; throws when the ABI is missing from the APK. */
    static synchronized void load(File nativeDir) {
        if (loaded) {
            return;
        }
        File direct = new File(nativeDir, "libzakoai.so");
        if (direct.isFile()) {
            System.load(direct.getAbsolutePath());
            loaded = true;
            ProxyRunner.log("loaded native library: " + direct.getAbsolutePath());
            return;
        }
        System.loadLibrary("zakoai");
        loaded = true;
        ProxyRunner.log("loaded native library via loadLibrary");
    }

    /** Blocks until the relay stops; safe to call from any background thread. */
    static void serve(String dataDir, String webDir, String bind) {
        nativeServe(dataDir, webDir, bind);
    }

    private static native void nativeServe(String dataDir, String webDir, String bind);
}
