#!/bin/bash
# lab/scripts/lab-build.sh
# Build sbconfig inside the Docker container (no cross-compilation needed)
set -e

cd "$(dirname "$0")/../.."

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

echo "Copying source code to container..."
docker exec sbconfig-lab mkdir -p /build
docker cp Cargo.toml sbconfig-lab:/build/
docker cp src sbconfig-lab:/build/

echo "Installing Rust in container (if needed)..."
docker exec sbconfig-lab bash -c '
    if ! command -v cargo &> /dev/null; then
        echo "Installing Rust..."
        curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    fi
'

echo "Building sbconfig inside container..."
docker exec sbconfig-lab bash -c '
    source ~/.cargo/env 2>/dev/null || true
    cd /build
    cargo build --release
'

echo "Installing binary..."
docker exec sbconfig-lab cp /build/target/release/sbconfig /usr/local/bin/
docker exec sbconfig-lab chmod +x /usr/local/bin/sbconfig

echo "Verifying installation..."
VERSION=$(docker exec sbconfig-lab /usr/local/bin/sbconfig --version 2>&1 || echo "unknown")
echo "Installed version: $VERSION"

echo ""
echo "Build and deployment complete!"
echo "Run: ./lab/scripts/lab-ssh.sh then 'sbconfig'"
