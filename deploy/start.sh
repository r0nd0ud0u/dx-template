#!/bin/bash
# Pull latest image and start the full stack in background.
# The db volume is preserved across restarts.
set -e
cd "$(dirname "$0")"

echo "Pulling latest images..."
docker compose pull

echo "Starting dx-template stack..."
docker compose up -d

echo ""
echo "  App       → http://localhost:8080"
echo "  SQLite UI → http://localhost:8082"
echo ""
echo "dx-template is starting — the SQLite UI waits for the app to be healthy (~20s)."
echo "Follow logs with: ./logs.sh"
