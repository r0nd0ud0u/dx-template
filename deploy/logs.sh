#!/bin/bash
# Tail live logs for all services (Ctrl+C to stop).
# Usage: ./logs.sh              — all services
#        ./logs.sh dx-template  — app only
#        ./logs.sh sqlite-web   — SQLite UI only
set -e
cd "$(dirname "$0")"

SERVICE="${1:-}"
docker compose logs -f --tail=50 $SERVICE
