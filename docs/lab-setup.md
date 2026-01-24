# Lab Setup

> Local development and testing environment for sbconfig.

## Overview

sbconfig development uses a **Development Mode** with `localhost` for testing. This allows you to:

- Develop on macOS and cross-compile for Linux
- Test the TUI without a real VPS
- Generate test configurations that work locally
- Validate all features before production deployment

## Development Workflow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Development Workflow                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   Mac (Development)                Linux (Testing)                       │
│   ─────────────────                ───────────────                       │
│   ├── Write Rust code              ├── Docker container OR               │
│   ├── cargo build (native)         ├── Linux VM OR                       │
│   ├── cross build (Linux)          └── Real VPS                          │
│   └── Run tests                                                          │
│                                                                          │
│   Development Mode                 Production Mode                       │
│   ────────────────                 ───────────────                       │
│   ├── Server: localhost            ├── Server: domain/IP                 │
│   ├── SSH port: any                ├── SSH port: custom (e.g., 7344)    │
│   └── Local testing only           └── Real deployment                   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Prerequisites

### macOS Development Machine

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cross for cross-compilation
cargo install cross

# Install Docker (required for cross)
# Download from https://www.docker.com/products/docker-desktop/
```

### Optional: Linux Testing Environment

For integration testing with actual SSH and systemd:

**Option A: Docker Container**
```bash
# Run Ubuntu container with systemd (requires privileged mode)
docker run -d --name sbconfig-test \
  --privileged \
  -v /sys/fs/cgroup:/sys/fs/cgroup:rw \
  ubuntu:22.04 /sbin/init
```

**Option B: Linux VM (UTM/Parallels)**
- Create Ubuntu 22.04 VM
- Snapshot clean state for easy reset

**Option C: Real VPS**
- Use for final testing before release
- Test production mode with real domain

## Development Mode Configuration

When running sbconfig in development mode:

### Settings Used
| Setting | Development | Production |
|---------|-------------|------------|
| Server Address | `localhost` or `127.0.0.1` | Domain or public IP |
| SSH Port | Any available port | Custom port (10000-60000) |
| Config Output | Works for local testing | Works for real clients |

### Example Development Config

Generated client config in development mode:

```json
{
  "outbounds": [
    {
      "type": "ssh",
      "tag": "ssh-out",
      "server": "localhost",
      "server_port": 2222,
      "user": "test_user",
      "private_key": "..."
    }
  ]
}
```

## Local Testing Setup

### 1. Set Up Test SSH Server (macOS)

macOS has built-in SSH server:

```bash
# Enable Remote Login in System Preferences > Sharing
# Or via command line:
sudo systemsetup -setremotelogin on

# Create test user (optional, can use your own user)
sudo dscl . -create /Users/testuser
sudo dscl . -create /Users/testuser UserShell /bin/bash
sudo dscl . -create /Users/testuser UniqueID 1001
sudo dscl . -create /Users/testuser PrimaryGroupID 80
sudo dscl . -create /Users/testuser NFSHomeDirectory /Users/testuser
sudo mkdir /Users/testuser
sudo chown testuser:staff /Users/testuser
```

### 2. Run sbconfig in Development Mode

```bash
# Build and run
cargo run

# On first run, select:
# - Mode: Development
# - Server: localhost
# - SSH Port: 22 (or custom)
```

### 3. Test Generated Configs

```bash
# Install sing-box on macOS
brew install sing-box

# Run with generated config
sing-box run -c /path/to/generated/config.json

# Test proxy
curl -x socks5://127.0.0.1:10808 https://httpbin.org/ip
```

## Cross-Compilation for Linux

### Build for Linux x86_64

```bash
# Using cross (recommended)
cross build --release --target x86_64-unknown-linux-musl

# Binary location
ls target/x86_64-unknown-linux-musl/release/sbconfig
```

### Build for Linux ARM64 (e.g., Oracle Cloud free tier)

```bash
cross build --release --target aarch64-unknown-linux-musl
```

### Deploy to VPS

```bash
# Copy binary to VPS
scp target/x86_64-unknown-linux-musl/release/sbconfig user@your-vps:/tmp/

# SSH to VPS and install
ssh user@your-vps
sudo mv /tmp/sbconfig /usr/local/bin/
sudo chmod +x /usr/local/bin/sbconfig

# Run (requires root for user management)
sudo sbconfig
```

## Testing Checklist

### Unit Tests (Local)

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

### Integration Tests (Linux)

These tests require a Linux environment:

- [ ] First-run wizard completes successfully
- [ ] sing-box detection works
- [ ] SSH port configuration saves correctly
- [ ] User creation creates system user
- [ ] SSH key generation produces valid keys
- [ ] Generated configs are valid JSON
- [ ] Generated configs work with sing-box

### Manual Testing Workflow

1. **Start fresh**
   ```bash
   # Remove existing database
   sudo rm -rf /var/lib/sbconfig/
   
   # Run sbconfig
   sudo sbconfig
   ```

2. **Complete first-run wizard**
   - Verify sing-box detection
   - Configure SSH port
   - Set to Development mode

3. **Create test user**
   - Navigate to User Management
   - Add new user
   - Verify system user created: `id username`

4. **Generate config**
   - Select the test user
   - Generate JSON config
   - Verify config is valid JSON

5. **Test config (if sing-box installed)**
   ```bash
   sing-box check -c /path/to/config.json
   sing-box run -c /path/to/config.json
   ```

## Docker-Based Testing

### Dockerfile for Testing

```dockerfile
# Dockerfile.test
FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    openssh-server \
    curl \
    sudo \
    && rm -rf /var/lib/apt/lists/*

# Set up SSH
RUN mkdir /var/run/sshd
RUN echo 'PermitRootLogin yes' >> /etc/ssh/sshd_config

# Install sing-box
RUN bash <(curl -fsSL https://sing-box.app/deb-install.sh)

# Create data directory
RUN mkdir -p /var/lib/sbconfig

# Copy binary (build first with cross)
COPY target/x86_64-unknown-linux-musl/release/sbconfig /usr/local/bin/

EXPOSE 22 2222

CMD ["/usr/sbin/sshd", "-D"]
```

### Build and Run Test Container

```bash
# Build Linux binary first
cross build --release --target x86_64-unknown-linux-musl

# Build test image
docker build -f Dockerfile.test -t sbconfig-test .

# Run container
docker run -d --name sbconfig-lab \
  -p 2222:22 \
  sbconfig-test

# Access container
docker exec -it sbconfig-lab bash

# Run sbconfig inside container
sbconfig
```

## Troubleshooting

### Cross-compilation fails

```bash
# Make sure Docker is running
docker ps

# Pull the cross image manually
docker pull ghcr.io/cross-rs/x86_64-unknown-linux-musl:latest
```

### SSH connection refused in development

```bash
# Check SSH is running
sudo lsof -i :22

# Check firewall
sudo pfctl -s rules
```

### sing-box not found

```bash
# Install sing-box on macOS
brew install sing-box

# Or download manually
# https://github.com/SagerNet/sing-box/releases
```

## Related Documentation

- [Development](./development.md) - Full development setup
- [Architecture](./architecture.md) - System design
- [Features](./features.md) - Feature specifications
- [Routing](./routing.md) - Traffic routing configuration
