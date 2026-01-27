#!/bin/bash
# lab/scripts/lab-rebuild.sh
# Force rebuild the lab container with latest Dockerfile changes
set -e

cd "$(dirname "$0")/.."

echo "Stopping and removing existing lab container..."
docker compose down

echo "Rebuilding lab image from scratch..."
docker compose build --no-cache

echo "Starting fresh lab container..."
./scripts/lab-start.sh

echo ""
echo "Lab rebuilt successfully!"
echo "Rust is now pre-installed in the image."
