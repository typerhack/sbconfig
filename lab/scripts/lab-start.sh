#!/bin/bash
# lab/scripts/lab-start.sh
# Start the sbconfig lab environment
set -e

cd "$(dirname "$0")/.."

echo "Starting sbconfig lab environment..."
# Check if we need to rebuild (use --build flag to force rebuild)
if [ "$1" == "--build" ]; then
    echo "Rebuilding lab image..."
    docker compose up -d --build
else
    # Check if container exists and is using old image without Rust
    if docker ps -a | grep -q sbconfig-lab; then
        if ! docker exec sbconfig-lab bash -c 'command -v cargo' &>/dev/null 2>&1; then
            echo "Rust not found in existing container. Rebuilding..."
            docker compose down
            docker compose up -d --build
        else
            docker compose up -d
        fi
    else
        # No container exists, build it
        docker compose up -d --build
    fi
fi

echo "Waiting for container to be ready..."
sleep 3

# Setup SSH key authentication
SSH_KEY="$HOME/.ssh/id_ed25519"
SSH_PUB="$HOME/.ssh/id_ed25519.pub"

# Generate SSH key if it doesn't exist
if [ ! -f "$SSH_KEY" ]; then
    echo "Generating SSH key..."
    ssh-keygen -t ed25519 -f "$SSH_KEY" -N "" -C "sbconfig-lab"
fi

# Copy public key to container for passwordless access
echo "Setting up SSH key authentication..."
docker exec sbconfig-lab mkdir -p /root/.ssh
docker exec sbconfig-lab chmod 700 /root/.ssh
docker cp "$SSH_PUB" sbconfig-lab:/root/.ssh/authorized_keys
docker exec sbconfig-lab chown root:root /root/.ssh/authorized_keys
docker exec sbconfig-lab chmod 600 /root/.ssh/authorized_keys

# Wait for SSH to be actually ready
echo "Waiting for SSH to be ready..."
for i in {1..10}; do
    if ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=2 -p 2222 root@localhost "exit" 2>/dev/null; then
        break
    fi
    echo "Waiting for SSH... (attempt $i/10)"
    sleep 2
done

echo ""
echo "Lab is ready!"
echo "  SSH:  ssh -p 2222 root@localhost"
echo "  Or:   ./scripts/lab-ssh.sh"
echo "  Stop: ./scripts/lab-stop.sh"
echo ""
