#!/bin/bash
# Bundles the desktop client (no server feature — talks to a remote server via SERVER_URL).
dx bundle --platform desktop --release --fullstack false --no-default-features --features desktop --out-dir bundle-desktop
