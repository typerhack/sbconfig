# sbconfig

A Rust-based TUI (Text User Interface) tool for managing sing-box SSH proxy configurations on Linux VPS servers.

> **Note:** sbconfig does NOT install sing-box. It detects if sing-box is installed and guides you to install it manually if needed.

## Features

- **One-Command Installation** - Install via `curl | bash` or download binary
- **First-Run Setup Wizard** - Guided configuration on first launch
- **Interactive TUI** - Beautiful terminal interface powered by ratatui
- **User Management** - Create, delete, and manage SSH proxy users
- **Auto Key Generation** - Automatic ED25519 SSH key pair generation
- **Client Config Export** - Generate configs for iOS, Android, Windows, macOS, Linux
- **QR Code Support** - Scan QR codes directly from terminal for mobile setup
- **sing-box Detection** - Detects sing-box installation and service status

## Quick Start

### Installation

**Method 1: One-liner (Recommended)**
```bash
curl -sSL https://raw.githubusercontent.com/YOUR_USERNAME/sbconfig/main/install.sh | sudo bash
```

**Method 2: Git Clone**
```bash
git clone https://github.com/YOUR_USERNAME/sbconfig.git
cd sbconfig
sudo ./install.sh
```

### First Run

```bash
sudo sbconfig
```

On first run, the **Setup Wizard** will guide you through:

1. **Operating Mode** - Choose Development (localhost) or Production (domain/IP)
2. **Server Address** - Enter your domain or public IP (production mode)
3. **SSH Port** - Configure a custom SSH port for proxy users (NOT port 22)
4. **Firewall** - Instructions to open the custom port
5. **sing-box Check** - Detection and installation guidance if needed

### Usage

```bash
# Run the TUI
sudo sbconfig

# Show help
sbconfig --help

# Show version
sbconfig --version
```

## Screenshots

```
┌──────────────────────────────────────────────────────────────────┐
│                    sbconfig - Main Dashboard                      │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│   ╔═══════════════════════════════════════════════════════════╗  │
│   ║  System Status                                             ║  │
│   ╠═══════════════════════════════════════════════════════════╣  │
│   ║  sing-box    │ ✓ Installed │ v1.10.0     │ ● Running      ║  │
│   ║  SSH Server  │ ✓ Active    │ Port: 7344  │ ● Listening    ║  │
│   ║  Database    │ ✓ OK        │ 3 users     │ 12 configs     ║  │
│   ╚═══════════════════════════════════════════════════════════╝  │
│                                                                   │
│   ┌─ Main Menu ─────────────────────────────────────────────────┐│
│   │  > [1] sing-box Status                                      ││
│   │    [2] User Management                                      ││
│   │    [3] Generate Client Config                               ││
│   │    [4] Server Settings                                      ││
│   │    [5] View Logs                                            ││
│   │    [q] Quit                                                 ││
│   └──────────────────────────────────────────────────────────────┘│
│                                                                   │
│   [↑/↓] Navigate  [Enter] Select  [q] Quit  [?] Help             │
└──────────────────────────────────────────────────────────────────┘
```

## Requirements

- **OS**: Linux (Ubuntu 20.04+, Debian 11+, CentOS 8+, or similar)
- **Privileges**: Root access required for user management
- **Dependencies**: None (static binary)
- **sing-box**: Must be installed separately (sbconfig will guide you)

## How It Works

### Architecture

```
┌─────────────┐      SSH Tunnel       ┌─────────────┐      Direct      ┌──────────────┐
│   Client    │ ────────────────────  │   Your VPS  │ ────────────────│   Internet   │
│ (sing-box)  │  Custom Port (7344)   │  (OpenSSH)  │                  │              │
└─────────────┘                       └─────────────┘                  └──────────────┘
```

### Key Concepts

| Concept | Description |
|---------|-------------|
| **Custom SSH Port** | Proxy users connect on a dedicated port (e.g., 7344), NOT port 22 |
| **Port 22 Reserved** | Standard SSH port remains for admin access only |
| **No Interactive Login** | Proxy users have `/sbin/nologin` shell - tunnel only |
| **Embedded Keys** | Private keys embedded in client configs for easy import |

### Operating Modes

| Mode | Server Address | Use Case |
|------|---------------|----------|
| **Development** | `localhost` | Local testing on Mac/Linux |
| **Production** | Domain or public IP | Real VPS deployment |

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](docs/architecture.md) | System design and module structure |
| [Features](docs/features.md) | Complete feature specifications |
| [Database](docs/database.md) | SQLite schema documentation |
| [UI Screens](docs/ui-screens.md) | TUI mockups and navigation |
| [Installation](docs/installation.md) | Detailed installation guide |
| [Configuration](docs/configuration.md) | Config generation details |
| [Routing](docs/routing.md) | Traffic routing configuration |
| [Development](docs/development.md) | Contributing and dev setup |
| [Lab Setup](docs/lab-setup.md) | Local development environment |

## Supported Platforms

### Server
- Ubuntu 20.04, 22.04, 24.04
- Debian 11, 12
- CentOS 8, 9
- Other Linux with systemd

### Client Configs
- iOS (sing-box app)
- Android (sing-box app)
- Windows
- macOS
- Linux

## Security

- **Dedicated SSH Port**: Proxy users connect on a custom port, separate from admin SSH
- **No Shell Access**: Proxy users cannot login interactively (`/sbin/nologin`)
- **Unique Keys**: Each user gets a unique ED25519 key pair
- **Embedded Keys**: Private keys embedded in client configs for easy mobile import
- **Local Storage**: Keys stored encrypted in the local SQLite database
- **No Passwords**: No passwords transmitted over the network

## sing-box Installation

sbconfig does NOT install sing-box automatically. When you run sbconfig, it will:

1. Check if sing-box is installed (`which sing-box`)
2. If not found, display installation instructions
3. Link to official sing-box documentation

**Official sing-box installation:**
- [sing-box Installation Guide](https://sing-box.sagernet.org/installation/)
- [sing-box GitHub](https://github.com/SagerNet/sing-box)

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read [docs/development.md](docs/development.md) for guidelines.

## Acknowledgments

- [sing-box](https://github.com/SagerNet/sing-box) - The universal proxy platform
- [ratatui](https://github.com/ratatui-org/ratatui) - Rust TUI library
