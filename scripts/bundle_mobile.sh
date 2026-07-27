#!/bin/bash
# Bundles the Android client (no server feature — talks to a remote server via SERVER_URL).
# --package-types apk forces a directly-installable .apk instead of dx's default .aab
# (Play Store bundle format), which CI can't sideload/verify as easily.
#
# Usage: ./scripts/bundle_mobile.sh [rustc-target-triple]
# The target triple must be explicit — without --target, dx picks its own default,
# which is not guaranteed to match the device you're installing on and produces an
# APK that silently fails to install ("app not compatible with this device").
# Defaults to aarch64-linux-android (arm64-v8a), the ABI on the vast majority of
# real Android phones since ~2019. x86_64-linux-android also works, for emulator
# testing. 32-bit targets (armv7-linux-androideabi, i686-linux-android) do NOT work
# as of dioxus 0.7.9 — its manganis crate hard-errors with "Only 64-bit Android
# targets are supported".
TARGET="${1:-aarch64-linux-android}"
dx bundle --platform android --release --fullstack false --package-types apk --target "$TARGET" --no-default-features --features mobile --out-dir bundle-android
