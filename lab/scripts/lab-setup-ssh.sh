#!/bin/bash
# lab/scripts/lab-setup-ssh.sh
# Setup SSH key-based authentication to the lab container
set -e

cd "$(dirname "$0")/.."

SSH_KEY="$HOME/.ssh/id_ed25519"
SSH_PUB="$HOME/.ssh/id_ed25519.pub"

# Generate SSH key if it doesn't exist
if [ ! -f "$SSH_KEY" ]; then
    echo "Generating SSH key..."
    ssh-keygen -t ed25519 -f "$SSH_KEY" -N "" -C "sbconfig-lab"
fi

# Check if container is running
if ! docker ps | grep -q sbconfig-lab; then
    echo "Error: Lab container is not running."
    echo "Start it with: ./scripts/lab-start.sh"
    exit 1
fi

# Copy public key to container
echo "Setting up SSH key authentication..."
docker exec sbconfig-lab mkdir -p /root/.ssh
docker exec sbconfig-lab chmod 700 /root/.ssh
docker cp "$SSH_PUB" sbconfig-lab:/root/.ssh/authorized_keys
docker exec sbconfig-lab chown root:root /root/.ssh/authorized_keys
docker exec sbconfig-lab chmod 600 /root/.ssh/authorized_keys

echo ""
echo "SSH key authentication configured!"
echo "You can now connect with: ssh -p 2222 root@localhost"
echo "Or use: ./scripts/lab-ssh.sh"
