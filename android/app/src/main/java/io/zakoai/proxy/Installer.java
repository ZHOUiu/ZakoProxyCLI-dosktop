package io.zakoai.proxy;

import android.content.Context;
import android.content.res.AssetManager;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

/** Extracts the bundled web console into app-private storage, once per version. */
final class Installer {
    private static final String STAMP = ".web-v0.13.2-r2";

    private Installer() {}

    static File dataDir(Context context) {
        File dir = new File(context.getFilesDir(), "data");
        if (!dir.isDirectory() && !dir.mkdirs()) {
            android.util.Log.e("ZakoAI", "cannot create " + dir);
        }
        return dir;
    }

    static File webDir(Context context) {
        return new File(context.getFilesDir(), "web");
    }

    static File serverBin(Context context) {
        return new File(context.getApplicationInfo().nativeLibraryDir, "libzakoai.so");
    }

    static synchronized void syncAssets(Context context) throws IOException {
        File web = webDir(context);
        File stamp = new File(context.getFilesDir(), STAMP);
        if (stamp.isFile() && entryMatches(web)) {
            return;
        }
        deleteTree(web);
        if (!web.mkdirs() && !web.isDirectory()) {
            throw new IOException("cannot create " + web.getAbsolutePath());
        }
        copyTree(context.getAssets(), "web", web);
        try (OutputStream out = new FileOutputStream(stamp)) {
            out.write(BuildInfo.VERSION.getBytes("UTF-8"));
        }
        android.util.Log.i("ZakoAI", "console assets extracted to " + web);
    }

    /** The extracted index.html must reference assets that are actually on disk. */
    private static boolean entryMatches(File web) {
        File index = new File(web, "index.html");
        if (!index.isFile()) {
            return false;
        }
        try {
            String html = new String(
                    java.nio.file.Files.readAllBytes(index.toPath()), java.nio.charset.StandardCharsets.UTF_8);
            java.util.regex.Matcher matcher = java.util.regex.Pattern
                    .compile("/assets/([A-Za-z0-9_.-]+[.]js)")
                    .matcher(html);
            while (matcher.find()) {
                if (!new File(new File(web, "assets"), matcher.group(1)).isFile()) {
                    return false;
                }
            }
            return true;
        } catch (Exception e) {
            return false;
        }
    }

    private static void copyTree(AssetManager assets, String path, File target) throws IOException {
        String[] children;
        try {
            children = assets.list(path);
        } catch (IOException e) {
            children = null;
        }
        if (children == null || children.length == 0) {
            try (InputStream in = assets.open(path); OutputStream out = new FileOutputStream(target)) {
                byte[] buffer = new byte[128 * 1024];
                int read;
                while ((read = in.read(buffer)) != -1) {
                    out.write(buffer, 0, read);
                }
            }
            return;
        }
        if (!target.mkdirs() && !target.isDirectory()) {
            throw new IOException("cannot create " + target.getAbsolutePath());
        }
        for (String child : children) {
            copyTree(assets, path + "/" + child, new File(target, child));
        }
    }

    private static void deleteTree(File file) {
        File[] children = file.listFiles();
        if (children != null) {
            for (File child : children) {
                deleteTree(child);
            }
        }
        //noinspection ResultOfMethodCallIgnored
        file.delete();
    }
}
