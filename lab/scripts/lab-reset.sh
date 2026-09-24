#!/bin/bash
# lab/scripts/lab-reset.sh
# Reset the sbconfig lab to a clean state
cd "$(dirname "$0")/.."

echo "Resetting sbconfig lab to clean state..."
echo "This will remove all data and rebuild the container."
read -p "Continue? [y/N] " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    docker compose down -v
    docker compose up -d --build
    sleep 5
    echo "Lab reset complete!"
    echo "  SSH: ssh -p 2222 root@localhost (password: labpass)"
else
    echo "Reset cancelled."
fi
