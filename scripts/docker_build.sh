#!/bin/bash
# Build the Docker image locally.
# --no-cache ensures all layers are rebuilt fresh (no stale cached layers).
# After a successful build, dangling images are pruned automatically.
GIT_DESCRIBE=$(git describe --dirty --tags 2>/dev/null || echo "dev")
docker buildx build \
  --no-cache \
  --tag dx-template:latest \
  --build-arg GIT_DESCRIBE="$GIT_DESCRIBE" \
  .

echo "Pruning dangling images..."
docker image prune -f
echo "Build complete: dx-template:latest"
