#!/bin/bash
# Runs the Android native client and points it at dev_web.sh's server.
#
# Uses `adb reverse` so the device/emulator's own 127.0.0.1:8080 tunnels straight to the dev
# machine's 127.0.0.1:8080 — this works for both the emulator and a real device over USB, and
# (unlike the 10.0.2.2 emulator alias) it also satisfies Android's default WebView network
# security config, which only permits cleartext (plain http, no TLS) traffic to 127.0.0.1. That
# matters because <img> tags are rendered by the WebView (subject to that policy) while
# server-fn/websocket calls go through Rust's own reqwest client (not subject to it) — so
# without the reverse tunnel, character images silently fail to load even though login and
# gameplay work fine.
#
# Usage:
#   ./scripts/dev_android.sh                                   # emulator or real device over adb
#   SERVER_URL=http://<lan-ip>:8080 ./scripts/dev_android.sh   # real device without adb reverse (e.g. wifi-adb on a different subnet)

# `adb` isn't always on PATH (e.g. VS Code's git-bash task shell on Windows) even when
# ANDROID_HOME/ANDROID_SDK_ROOT is set, so fall back to locating it under the SDK root.
if command -v adb >/dev/null 2>&1; then
    ADB=adb
elif [ -n "$ANDROID_HOME" ] && [ -x "$ANDROID_HOME/platform-tools/adb.exe" ]; then
    ADB="$ANDROID_HOME/platform-tools/adb.exe"
elif [ -n "$ANDROID_HOME" ] && [ -x "$ANDROID_HOME/platform-tools/adb" ]; then
    ADB="$ANDROID_HOME/platform-tools/adb"
elif [ -n "$ANDROID_SDK_ROOT" ] && [ -x "$ANDROID_SDK_ROOT/platform-tools/adb.exe" ]; then
    ADB="$ANDROID_SDK_ROOT/platform-tools/adb.exe"
elif [ -n "$ANDROID_SDK_ROOT" ] && [ -x "$ANDROID_SDK_ROOT/platform-tools/adb" ]; then
    ADB="$ANDROID_SDK_ROOT/platform-tools/adb"
else
    ADB=""
fi

if [ -n "$ADB" ]; then
    "$ADB" reverse tcp:8080 tcp:8080 || echo "warning: adb reverse failed — is a device/emulator connected? ($ADB devices)"
else
    echo "warning: adb not found (checked PATH, \$ANDROID_HOME/platform-tools, \$ANDROID_SDK_ROOT/platform-tools) — skipping adb reverse. Set ANDROID_HOME or add platform-tools to PATH, or images/reconnect may fail."
fi
SERVER_URL="${SERVER_URL:-http://127.0.0.1:8080}" dx serve --platform android --no-default-features --features mobile
