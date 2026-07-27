#!/bin/bash
# Runs the desktop native client. Defaults to SERVER_URL=http://127.0.0.1:8080,
# so it talks to dev_web.sh on this same machine with no extra setup.
# Point at a different server: SERVER_URL=https://your-server.example.com ./scripts/dev_desktop.sh
dx serve --platform desktop --no-default-features --features desktop
