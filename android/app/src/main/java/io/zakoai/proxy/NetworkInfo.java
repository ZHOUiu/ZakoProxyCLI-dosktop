package io.zakoai.proxy;

import android.content.Context;
import android.net.wifi.WifiManager;
import android.text.format.Formatter;

import java.io.File;
import java.io.IOException;
import java.net.InetAddress;
import java.net.NetworkInterface;
import java.util.Collections;

/** Small helpers for the launcher screen. */
final class NetworkInfo {
    private NetworkInfo() {}

    static String lanAddress(Context context) {
        try {
            for (NetworkInterface nif : Collections.list(NetworkInterface.getNetworkInterfaces())) {
                if (!nif.isUp() || nif.isLoopback()) {
                    continue;
                }
                String name = nif.getName();
                if (!name.startsWith("wlan") && !name.startsWith("eth") && !name.startsWith("ap")) {
                    continue;
                }
                for (InetAddress address : Collections.list(nif.getInetAddresses())) {
                    String host = address.getHostAddress();
                    if (host != null && host.indexOf(':') < 0 && !host.startsWith("127.")) {
                        return host;
                    }
                }
            }
        } catch (Exception ignored) {
        }
        try {
            WifiManager wifi = (WifiManager) context.getApplicationContext()
                    .getSystemService(Context.WIFI_SERVICE);
            if (wifi != null) {
                @SuppressWarnings("deprecation")
                int ip = wifi.getConnectionInfo().getIpAddress();
                if (ip != 0) {
                    @SuppressWarnings("deprecation")
                    String text = Formatter.formatIpAddress(ip);
                    return text;
                }
            }
        } catch (Exception ignored) {
        }
        return null;
    }

    static int portOf(String bind) {
        int colon = bind.lastIndexOf(':');
        try {
            return Integer.parseInt(bind.substring(colon + 1));
        } catch (Exception e) {
            return 1990;
        }
    }

    static String size(File file) {
        long bytes = file.exists() ? file.length() : 0;
        if (bytes > 1024 * 1024) {
            return String.format("%.1f MB", bytes / 1048576.0);
        }
        return String.format("%.0f KB", bytes / 1024.0);
    }

    static String readTail(File file, int lines) throws IOException {
        if (!file.isFile()) {
            return "";
        }
        StringBuilder out = new StringBuilder();
        int count = 0;
        try (java.io.RandomAccessFile raf = new java.io.RandomAccessFile(file, "r")) {
            long position = raf.length();
            java.util.ArrayList<String> collected = new java.util.ArrayList<>();
            while (position > 0 && collected.size() <= lines) {
                long step = Math.min(4096, position);
                position -= step;
                raf.seek(position);
                byte[] buffer = new byte[(int) step];
                raf.readFully(buffer);
                String chunk = new String(buffer, "UTF-8");
                String[] parts = chunk.split("\n");
                for (int i = parts.length - 1; i >= 0; i--) {
                    if (!parts[i].isEmpty()) {
                        collected.add(parts[i]);
                    }
                    if (collected.size() > lines) {
                        break;
                    }
                }
            }
            for (int i = collected.size() - 1; i >= 0 && count < lines; i--, count++) {
                out.append(collected.get(i)).append('\n');
            }
        }
        return out.toString();
    }
}
