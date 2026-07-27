#!/bin/bash
# Stop the stack. The db volume is preserved — data is NOT lost.
# To fully reset data: docker compose down -v
set -e
cd "$(dirname "$0")"

docker compose down
echo "Stack stopped. Volume preserved. Run ./start.sh to restart."
