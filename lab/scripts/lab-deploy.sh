#!/bin/bash
# lab/scripts/lab-deploy.sh
# Build sbconfig for Linux and deploy to the lab container
set -e

cd "$(dirname "$0")/../.."

# Check if cross is installed
if ! command -v cross &> /dev/null; then
    echo "Error: 'cross' is not installed."
    echo "Install it with: cargo install cross"
    exit 1
fi

# Check if Docker is running
if ! docker ps &> /dev/null; then
    echo "Error: Docker is not running."
    exit 1
fi

# Check if lab container is running
if ! docker ps | grep -q sbconfig-lab; then
    echo "Error: Lab container is not running."
    echo "Start it with: ./lab/scripts/lab-start.sh"
    exit 1
fi

TARGET="x86_64-unknown-linux-musl"
EXTRA_FLAGS=""
HOST_ARCH="$(uname -m)"
if [[ "$HOST_ARCH" != "x86_64" ]]; then
    EXTRA_FLAGS="--force-non-host"
fi

echo "Building sbconfig for Linux ($TARGET)..."
cross build --release --target "$TARGET" $EXTRA_FLAGS

echo "Deploying to lab container..."
docker cp "target/$TARGET/release/sbconfig" sbconfig-lab:/usr/local/bin/

echo "Setting permissions..."
docker exec sbconfig-lab chmod +x /usr/local/bin/sbconfig

echo "Verifying installation..."
VERSION=$(docker exec sbconfig-lab /usr/local/bin/sbconfig --version 2>&1 || echo "unknown")
echo "Installed version: $VERSION"

echo ""
echo "Deployment complete!"
echo "Run: ./lab/scripts/lab-ssh.sh then 'sbconfig'"
