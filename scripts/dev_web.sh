#!/bin/bash
# Runs the fullstack web server (WASM client + Axum server) as the main dev server.
# IP=0.0.0.0 makes it reachable from other devices on the LAN (e.g. a real Android
# phone running dev_android.sh with a LAN SERVER_URL) instead of only from this machine.
IP=0.0.0.0 IS_MAIN_SERVER=true dx serve --platform web
