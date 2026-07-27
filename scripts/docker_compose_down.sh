#!/bin/bash
# Stop the Docker Compose stack.
# The db volume is preserved — data is NOT lost.
#
# To also remove volumes (full reset): docker compose down -v

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_DIR="$SCRIPT_DIR/../deploy"

echo "Stopping dx-template stack..."
docker-compose -f "$COMPOSE_DIR/docker-compose.yml" down --remove-orphans
echo "Done. Volumes are preserved (data not lost)."
echo "To fully reset volumes: docker-compose -f deploy/docker-compose.yml down -v"
