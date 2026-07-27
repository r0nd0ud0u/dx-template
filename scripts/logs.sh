#!/bin/bash
# Follow live logs for the dx-template app container.
# Pass a service name as argument to see logs for a specific service.
# Examples:
#   ./scripts/logs.sh             → logs for dx-template only
#   ./scripts/logs.sh sqlite-web  → logs for sqlite-web

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

SERVICE="${1:-dx-template}"

echo "Following logs for '$SERVICE' (Ctrl+C to stop)..."
echo ""
docker compose logs -f --tail=50 "$SERVICE"
