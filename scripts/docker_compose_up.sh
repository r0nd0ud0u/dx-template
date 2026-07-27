#!/bin/bash
# Start the full stack (app + sqlite-web) with Docker Compose.
# Data is persisted in named Docker volumes across stop/start cycles.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_DIR="$SCRIPT_DIR/../deploy"

VOLUME_NAME="dx-template_db_data"

# Stop any running containers first
echo "Stopping any existing containers..."
docker-compose -f "$COMPOSE_DIR/docker-compose.yml" down 2>/dev/null || true

# Ensure /data/db.sqlite exists inside the volume before starting the app.
echo "Ensuring database file exists in volume '$VOLUME_NAME'..."
docker run --rm \
  -v "${VOLUME_NAME}:/data" \
  alpine sh -c "mkdir -p /data && touch /data/db.sqlite && chmod 666 /data/db.sqlite"
echo "  /data/db.sqlite is ready."

echo "Starting dx-template stack..."
docker-compose -f "$COMPOSE_DIR/docker-compose.yml" up -d --remove-orphans

echo ""
echo "Services:"
echo "  App        → http://localhost:8080"
echo "  SQLite UI  → http://localhost:8082  (loopback only — SSH tunnel for remote access)"
echo ""
echo "To follow logs:  ./logs.sh"
echo "To stop:         ./docker_compose_down.sh"