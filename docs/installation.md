# Installation

> Installation methods and scripts for sbconfig.

## Quick Install

### Method 1: One-liner (Recommended)

```bash
curl -sSL https://raw.githubusercontent.com/YOUR_USERNAME/sbconfig/main/install.sh | sudo bash
```

### Method 2: Git Clone

```bash
git clone https://github.com/YOUR_USERNAME/sbconfig.git
cd sbconfig
sudo ./install.sh
```

### Method 3: Manual Download

```bash
# Download the latest release
wget https://github.com/YOUR_USERNAME/sbconfig/releases/latest/download/sbconfig-linux-amd64.tar.gz

# Extract
tar -xzf sbconfig-linux-amd64.tar.gz

# Install
sudo mv sbconfig /usr/local/bin/
sudo chmod +x /usr/local/bin/sbconfig

# Create data directory
sudo mkdir -p /var/lib/sbconfig
sudo chmod 700 /var/lib/sbconfig
```

## System Requirements

### Minimum Requirements

| Requirement | Specification |
|-------------|---------------|
| OS | Linux (kernel 4.4+) |
| Architecture | x86_64 or aarch64 |
| RAM | 64 MB free |
| Disk | 50 MB free |
| Privileges | Root access (sudo) |

### Supported Distributions

| Distribution | Versions | Status |
|--------------|----------|--------|
| Ubuntu | 20.04, 22.04, 24.04 | Fully Supported |
| Debian | 11, 12 | Fully Supported |
| CentOS | 8, 9 | Fully Supported |
| Rocky Linux | 8, 9 | Fully Supported |
| Fedora | 38, 39, 40 | Fully Supported |
| Arch Linux | Latest | Supported |
| Alpine | 3.18+ | Supported (musl) |

### Dependencies

sbconfig is distributed as a static binary with no runtime dependencies.

**Optional dependencies:**
- `systemd` - For service management (most distros)
- `ufw` or `firewalld` - For firewall configuration (manual)

**External requirement:**
- `sing-box` - Must be installed separately (sbconfig guides you through this)

## Installation Script

The installation script (`install.sh`) performs the following steps:

```bash
#!/bin/bash
set -e

# Configuration
REPO="YOUR_USERNAME/sbconfig"
INSTALL_DIR="/usr/local/bin"
DATA_DIR="/var/lib/sbconfig"
BINARY_NAME="sbconfig"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}sbconfig Installer${NC}"
echo "================================"

# Check root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: Please run as root (sudo)${NC}"
    exit 1
fi

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        ARCH="amd64"
        ;;
    aarch64|arm64)
        ARCH="arm64"
        ;;
    *)
        echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

echo "Detected architecture: $ARCH"

# Detect OS
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
    VERSION=$VERSION_ID
else
    echo -e "${RED}Error: Cannot detect OS${NC}"
    exit 1
fi

echo "Detected OS: $OS $VERSION"

# Get latest version
echo "Fetching latest version..."
LATEST_VERSION=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_VERSION" ]; then
    echo -e "${YELLOW}Warning: Could not fetch latest version, using 'latest'${NC}"
    LATEST_VERSION="latest"
fi

echo "Installing version: $LATEST_VERSION"

# Download binary
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_VERSION/sbconfig-linux-$ARCH.tar.gz"
echo "Downloading from: $DOWNLOAD_URL"

TMP_DIR=$(mktemp -d)
curl -sL "$DOWNLOAD_URL" -o "$TMP_DIR/sbconfig.tar.gz"

# Extract and install
cd "$TMP_DIR"
tar -xzf sbconfig.tar.gz

# Backup existing installation
if [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
    echo "Backing up existing installation..."
    mv "$INSTALL_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME.bak"
fi

# Install binary
mv sbconfig "$INSTALL_DIR/$BINARY_NAME"
chmod 755 "$INSTALL_DIR/$BINARY_NAME"

# Create data directory
mkdir -p "$DATA_DIR"
chmod 700 "$DATA_DIR"

# Cleanup
rm -rf "$TMP_DIR"

# Verify installation
if [ -x "$INSTALL_DIR/$BINARY_NAME" ]; then
    VERSION=$("$INSTALL_DIR/$BINARY_NAME" --version 2>/dev/null || echo "unknown")
    echo ""
    echo -e "${GREEN}Installation successful!${NC}"
    echo "================================"
    echo "Binary: $INSTALL_DIR/$BINARY_NAME"
    echo "Data:   $DATA_DIR"
    echo "Version: $VERSION"
    echo ""
    echo "Run 'sudo sbconfig' to start the setup wizard"
else
    echo -e "${RED}Error: Installation failed${NC}"
    exit 1
fi
```

## Post-Installation

### First Run - Setup Wizard

After installation, run sbconfig to start the **First-Run Setup Wizard**:

```bash
sudo sbconfig
```

The setup wizard will guide you through:

1. **Operating Mode Selection**
   - **Development**: Uses `localhost` as server address (for local testing)
   - **Production**: Uses your domain or public IP (for real VPS)

2. **Server Address** (Production mode only)
   - Enter your domain name or public IP address
   - This is what clients will connect to

3. **Custom SSH Port Configuration**
   - sbconfig uses a **dedicated custom port** for proxy users (NOT port 22)
   - Port 22 remains for admin SSH access only
   - If you already have a custom port configured, enter it
   - Otherwise, sbconfig will generate a random port (e.g., 7344)
   - The port is added to `/etc/ssh/sshd_config`

4. **Firewall Instructions**
   - You'll be shown commands to open the custom SSH port
   - Commands are provided for UFW, firewalld, and iptables

5. **sing-box Detection**
   - sbconfig checks if sing-box is installed
   - If not found, shows installation instructions
   - Links to official sing-box documentation

> **Important:** User management features are blocked until the setup wizard is completed.

### Firewall Configuration

After the wizard shows your custom SSH port, open it in your firewall:

**UFW (Ubuntu/Debian):**
```bash
# Replace 7344 with your actual custom port
sudo ufw allow 7344/tcp
sudo ufw reload
```

**firewalld (CentOS/Fedora/RHEL):**
```bash
# Replace 7344 with your actual custom port
sudo firewall-cmd --permanent --add-port=7344/tcp
sudo firewall-cmd --reload
```

**iptables (manual):**
```bash
# Replace 7344 with your actual custom port
sudo iptables -A INPUT -p tcp --dport 7344 -j ACCEPT
sudo iptables-save > /etc/iptables/rules.v4  # Debian/Ubuntu
# or
sudo service iptables save  # CentOS/RHEL
```

### sing-box Installation

sbconfig does **NOT** install sing-box automatically. You must install it manually.

**Official installation methods:**

1. **Package Manager** (recommended):
   ```bash
   # Ubuntu/Debian (add repository first)
   sudo apt install sing-box
   
   # Arch Linux
   sudo pacman -S sing-box
   ```

2. **Official Install Script**:
   ```bash
   bash <(curl -fsSL https://sing-box.app/deb-install.sh)  # Debian/Ubuntu
   bash <(curl -fsSL https://sing-box.app/rpm-install.sh)  # CentOS/Fedora
   ```

3. **Manual Download**:
   - Visit: https://github.com/SagerNet/sing-box/releases
   - Download the appropriate binary for your architecture

**Official Documentation:**
- [sing-box Installation Guide](https://sing-box.sagernet.org/installation/)
- [sing-box GitHub](https://github.com/SagerNet/sing-box)

## Updating sbconfig

### Method 1: Re-run Installer

```bash
curl -sSL https://raw.githubusercontent.com/YOUR_USERNAME/sbconfig/main/install.sh | sudo bash
```

### Method 2: Manual Update

```bash
# Download new version
wget https://github.com/YOUR_USERNAME/sbconfig/releases/latest/download/sbconfig-linux-amd64.tar.gz

# Backup database
sudo cp /var/lib/sbconfig/sbconfig.db /var/lib/sbconfig/sbconfig.db.bak

# Replace binary
sudo tar -xzf sbconfig-linux-amd64.tar.gz -C /usr/local/bin/
```

> **Note:** Your database and settings are preserved during updates.

## Uninstallation

### Full Uninstall

```bash
# Remove binary
sudo rm /usr/local/bin/sbconfig

# Remove data (WARNING: deletes all users and configs!)
sudo rm -rf /var/lib/sbconfig

# Remove any created system users (optional, manual)
# sudo userdel -r user_alpha
# sudo userdel -r user_beta
```

### Uninstall Script

```bash
#!/bin/bash

echo "sbconfig Uninstaller"
echo "===================="

read -p "Remove all data including users and configs? [y/N] " confirm

if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
    echo "Uninstall cancelled"
    exit 0
fi

# Remove binary
if [ -f /usr/local/bin/sbconfig ]; then
    sudo rm /usr/local/bin/sbconfig
    echo "Removed binary"
fi

# Remove data
if [ -d /var/lib/sbconfig ]; then
    sudo rm -rf /var/lib/sbconfig
    echo "Removed data directory"
fi

echo ""
echo "Uninstall complete"
echo "Note: System users created by sbconfig are NOT removed"
echo "Remove them manually with: sudo userdel -r <username>"
```

## File Locations

| Path | Purpose | Permissions |
|------|---------|-------------|
| `/usr/local/bin/sbconfig` | Main binary | `755` (root) |
| `/var/lib/sbconfig/` | Data directory | `700` (root) |
| `/var/lib/sbconfig/sbconfig.db` | SQLite database | `600` (root) |
| `/var/lib/sbconfig/configs/` | Generated configs | `700` (root) |
| `/var/lib/sbconfig/backups/` | Database backups | `700` (root) |

## Troubleshooting

### Permission Denied

```
Error: Permission denied
```

**Solution**: Run with sudo
```bash
sudo sbconfig
```

### Cannot Create User

```
Error: Failed to create system user
```

**Solution**: Check if user management tools are available
```bash
which useradd
which userdel
```

### Setup Not Complete

```
Error: Setup not complete. Please complete the setup wizard first.
```

**Solution**: Run sbconfig and complete the first-run setup wizard
```bash
sudo sbconfig
# Follow the setup wizard prompts
```

### Database Locked

```
Error: Database is locked
```

**Solution**: Only one instance can run at a time. Check for existing processes:
```bash
ps aux | grep sbconfig
```

### Architecture Not Supported

```
Error: Unsupported architecture
```

**Solution**: Check your architecture and ensure a compatible binary exists:
```bash
uname -m
# Should be x86_64 or aarch64
```

### sing-box Not Found

```
Warning: sing-box is not installed
```

**Solution**: Install sing-box manually following the official guide:
- https://sing-box.sagernet.org/installation/

### SSH Port Already in Use

```
Error: Port XXXX is already in use
```

**Solution**: Choose a different custom port or check what's using that port:
```bash
sudo ss -tlnp | grep XXXX
```

## Security Notes

1. **Run as Root**: sbconfig requires root privileges to manage system users
2. **Data Protection**: The database is only readable by root (`chmod 600`)
3. **Private Keys**: Stored in the database (consider encryption at rest)
4. **Backup Security**: Backups contain keys - protect them!
5. **Custom SSH Port**: Adds security through obscurity, separate from admin SSH
6. **No Shell Access**: Proxy users have `/sbin/nologin` - tunnel only, no interactive login

## Related Documentation

- [Architecture](./architecture.md) - System design
- [Features](./features.md) - Feature list
- [Database](./database.md) - Database schema
- [UI Screens](./ui-screens.md) - TUI mockups
- [Configuration](./configuration.md) - Config generation
- [Development](./development.md) - Building from source
- [Lab Setup](./lab-setup.md) - Local development environment
