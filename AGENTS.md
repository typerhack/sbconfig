# AGENTS.md - sbconfig Project Instructions

> This file provides context and instructions for AI agents working on the sbconfig project.

## Project Overview

**sbconfig** is a Rust-based TUI (Text User Interface) tool for managing sing-box SSH proxy configurations on Linux VPS servers. It manages SSH users, generates keys, and creates client configurations.

> **Important:** sbconfig does NOT install sing-box. It only detects if sing-box is installed and guides users to install manually if needed.

## Quick Links to Documentation

| Document | Description |
|----------|-------------|
| [README.md](./README.md) | Project overview, quick start, and basic usage |
| [docs/architecture.md](./docs/architecture.md) | System architecture and module structure |
| [docs/features.md](./docs/features.md) | Complete feature list and specifications |
| [docs/database.md](./docs/database.md) | SQLite schema and data models |
| [docs/ui-screens.md](./docs/ui-screens.md) | TUI screen mockups and navigation flow |
| [docs/installation.md](./docs/installation.md) | Installation methods and scripts |
| [docs/configuration.md](./docs/configuration.md) | sing-box config generation logic |
| [docs/routing.md](./docs/routing.md) | Traffic routing presets and custom rules |
| [docs/logging.md](./docs/logging.md) | Session-based logging system |
| [docs/development.md](./docs/development.md) | Development setup and guidelines |
| [docs/lab-setup.md](./docs/lab-setup.md) | Local development and testing environment |

## Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Language | Rust | Core application |
| TUI Framework | `ratatui` + `crossterm` | Terminal user interface |
| Database | SQLite via `rusqlite` | User and config storage |
| SSH Keys | `ssh-key` crate | ED25519/RSA key generation |
| QR Codes | `qrcode` crate | Mobile config sharing |
| Serialization | `serde` + `serde_json` | Config file handling |

## Project Structure

```
sbconfig/
├── AGENTS.md                 # This file - AI agent instructions
├── README.md                 # Project readme
├── Cargo.toml               # Rust dependencies
├── docs/                    # Documentation
│   ├── architecture.md      # System architecture
│   ├── features.md          # Feature specifications
│   ├── database.md          # Database schema
│   ├── ui-screens.md        # TUI mockups
│   ├── installation.md      # Installation guide
│   ├── configuration.md     # Config generation
│   ├── development.md       # Dev setup
│   └── lab-setup.md         # Local testing environment
├── src/
│   ├── main.rs              # Entry point
│   ├── app.rs               # App state management
│   ├── ui/                  # TUI components
│   │   ├── mod.rs
│   │   ├── home.rs          # Dashboard
│   │   ├── setup.rs         # First-run setup wizard
│   │   ├── status.rs        # sing-box status (detection only)
│   │   ├── users.rs         # User management
│   │   ├── configs.rs       # Config generation
│   │   ├── settings.rs      # Server settings
│   │   ├── logs.rs          # Log viewer
│   │   └── components.rs    # Reusable widgets
│   ├── db/                  # Database layer
│   │   ├── mod.rs
│   │   ├── schema.rs
│   │   └── queries.rs
│   ├── ssh/                 # SSH operations
│   │   ├── mod.rs
│   │   ├── keys.rs          # Key generation
│   │   └── users.rs         # System user management
│   ├── singbox/             # sing-box operations
│   │   ├── mod.rs
│   │   ├── detect.rs        # Detection (NOT installation)
│   │   ├── config.rs        # Config templates
│   │   └── service.rs       # systemd management
│   └── utils/               # Utilities
│       ├── mod.rs
│       └── system.rs        # System commands
├── install.sh               # Installation script
└── .github/
    └── workflows/
        └── release.yml      # CI/CD for releases
```

## Key Design Decisions

### 1. No Automatic Installation
sbconfig does NOT install sing-box automatically. It:
- Detects if sing-box is installed (`which sing-box`)
- Shows installation instructions if not found
- Links to official sing-box documentation
- This keeps the tool focused and avoids installation issues

### 2. Custom SSH Port (NOT Port 22)
sbconfig uses a **dedicated custom SSH port** for proxy users:
- Default SSH (port 22) is for admin access only
- sing-box proxy users connect on a custom port (e.g., 7344)
- First-run wizard asks: "Do you have a custom port?" → if no, generates one
- This separation improves security and management

### 3. Operating Modes
Two modes for different use cases:

| Mode | Server Address | Use Case |
|------|---------------|----------|
| **Development** | `localhost` | Local testing on Mac/Linux |
| **Production** | Domain or public IP | Real VPS deployment |

### 4. SSH-Only Protocol
The tool focuses exclusively on SSH protocol for sing-box:
- Works through most firewalls
- Uses custom port (NOT 22)
- No additional server-side software needed beyond OpenSSH
- Built-in encryption and authentication

### 5. SQLite for Storage
All user data and configurations are stored in SQLite:
- Single file database, easy to backup
- No external database server needed
- Good enough performance for this use case
- Built-in with Rust via `rusqlite` with bundled feature

### 6. Embedded Private Keys
Client configurations use embedded private keys (not file paths):
- Works on all platforms including iOS/Android
- Single config file contains everything needed
- Easier to share and import

### 7. Single Server Scope
The tool manages one server at a time (the one it's installed on):
- No remote SSH connections needed
- Direct system access for user creation
- Direct access to sing-box service

### 8. Restricted User Shell
Proxy users are created with `/sbin/nologin` as their shell:
- Users cannot log in interactively to the server
- SSH tunneling/port forwarding still works
- Improves security by preventing shell access
- Standard practice for service-only accounts

### 9. Traffic Routing Presets
Built-in routing configurations for common use cases:
- **Default**: All traffic through proxy
- **Iran Direct**: Iranian sites bypass proxy (using geosite:ir rule sets)
- **Iran Block**: Block Iranian government sites
- **China Direct**: Chinese sites bypass proxy (using geosite:cn rule sets)
- **Custom**: User-defined rules with domain, IP, port, and protocol matching
- Uses external rule sets from bootmortis/iran-hosted-domains and SagerNet/sing-geosite

## Implementation Guidelines

### Code Style
- Use Rust 2021 edition
- Follow Rust API guidelines
- Use `anyhow` for error handling in application code
- Use `thiserror` for library-style error types
- Prefer `async` operations where beneficial
- **NO EMOJIS**: Do not use emojis anywhere in the codebase, documentation, comments, UI text, error messages, or commit messages. Keep all text professional and plain ASCII.

### TUI Patterns
- Each screen is a separate module in `src/ui/`
- Use a central `App` struct for state management
- Navigation uses a screen stack for back functionality
- All screens implement a common `Screen` trait

### Database Operations
- All SQL queries in `src/db/queries.rs`
- Use prepared statements
- Transactions for multi-step operations
- Schema migrations for future updates

### Security Considerations
- Private keys stored encrypted in database
- Sensitive data cleared from memory after use
- No hardcoded credentials
- Validate all user input

## Common Tasks

### Adding a New Screen
1. Create new file in `src/ui/` (e.g., `new_screen.rs`)
2. Implement the `Screen` trait
3. Add to `src/ui/mod.rs`
4. Add navigation in `src/app.rs`

### Adding a Database Table
1. Update schema in `src/db/schema.rs`
2. Add migration if needed
3. Create queries in `src/db/queries.rs`
4. Add model struct if needed

### Adding a New sing-box Config Option
1. Update config templates in `src/singbox/config.rs`
2. Add UI elements for the option
3. Update database schema if persistent
4. Document in `docs/configuration.md`

## Testing

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test

# Test specific module
cargo test db::
cargo test ui::
```

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Cross-compile for different architectures
cross build --release --target x86_64-unknown-linux-musl
cross build --release --target aarch64-unknown-linux-musl
```

## References

- [sing-box Documentation](https://sing-box.sagernet.org/)
- [sing-box SSH Outbound](https://sing-box.sagernet.org/configuration/outbound/ssh/)
- [ratatui Documentation](https://ratatui.rs/)
- [rusqlite Documentation](https://docs.rs/rusqlite/)
