# sbconfig Lab Environment

Local Docker-based development and testing environment for sbconfig.

## Overview

The lab environment provides an Ubuntu 22.04 container with:
- OpenSSH server (for testing SSH proxy functionality)
- sing-box (pre-installed)
- SQLite (for database testing)
- User management tools (useradd, userdel)

## Prerequisites

On your development machine (Mac/Linux):

1. **Docker Desktop** - https://www.docker.com/products/docker-desktop
2. **Rust toolchain** - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
3. **cross** (for cross-compilation) - `cargo install cross`

## Quick Start

```bash
# Start the lab
./scripts/lab-start.sh

# SSH into the lab
./scripts/lab-ssh.sh
# Password: labpass

# Run tests to verify lab is working
./scripts/lab-test.sh
```

## Scripts

| Script | Description |
|--------|-------------|
| `lab-start.sh` | Build and start the lab container |
| `lab-stop.sh` | Stop the lab container |
| `lab-reset.sh` | Reset lab to clean state (removes all data) |
| `lab-ssh.sh` | SSH into the lab container as root |
| `lab-deploy.sh` | Cross-compile sbconfig and deploy to lab |
| `lab-test.sh` | Run lab environment verification tests |

## Development Workflow

### 1. Start the lab

```bash
cd lab
./scripts/lab-start.sh
```

### 2. Make code changes

Edit your Rust code in `src/`

### 3. Build and deploy

```bash
./scripts/lab-deploy.sh
```

This will:
- Cross-compile for `x86_64-unknown-linux-musl`
- Copy the binary to the lab container
- Make it executable

### 4. Test in lab

```bash
./scripts/lab-ssh.sh
# Now in the container:
sbconfig
```

### 5. Iterate

Repeat steps 2-4 as needed.

## Container Details

| Property | Value |
|----------|-------|
| Container name | `sbconfig-lab` |
| OS | Ubuntu 22.04 |
| SSH port | 2222 (host) -> 22 (container) |
| sing-box proxy port | 7344 (host) -> 7344 (container) |
| Root password | `labpass` |
| Data volume | `sbconfig-data` mounted at `/var/lib/sbconfig` |
| Logs volume | `sbconfig-logs` mounted at `/var/log/sbconfig` |

## Troubleshooting

### Container won't start

```bash
# Check Docker is running
docker ps

# Check for port conflicts
lsof -i :2222
lsof -i :7344

# View container logs
docker logs sbconfig-lab
```

### SSH connection refused

```bash
# Wait a few seconds after starting
sleep 5

# Check if sshd is running
docker exec sbconfig-lab ps aux | grep sshd
```

### sing-box not found

```bash
# Rebuild the container
./scripts/lab-reset.sh

# Or manually check
docker exec sbconfig-lab which sing-box
docker exec sbconfig-lab sing-box version
```

### Cross-compilation fails

```bash
# Make sure cross is installed
cargo install cross

# Make sure Docker is running
docker ps

# Try building manually
cross build --release --target x86_64-unknown-linux-musl
```

## Manual Docker Commands

```bash
# View running containers
docker ps

# View container logs
docker logs sbconfig-lab

# Execute command in container
docker exec sbconfig-lab <command>

# Copy file to container
docker cp <local-file> sbconfig-lab:<container-path>

# Copy file from container
docker cp sbconfig-lab:<container-path> <local-file>

# Rebuild without cache
docker compose build --no-cache
```

## Related Documentation

- [Lab Setup Details](../docs/lab-setup.md)
- [Development Guide](../docs/development.md)
- [Development Phases](../docs/phases.md)
