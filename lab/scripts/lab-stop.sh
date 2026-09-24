#!/bin/bash
# lab/scripts/lab-stop.sh
# Stop the sbconfig lab environment
cd "$(dirname "$0")/.."

echo "Stopping sbconfig lab..."
docker compose down
echo "Lab stopped."
