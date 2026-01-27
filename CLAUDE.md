# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**sbconfig** is a Rust TUI application for managing sing-box SSH proxy configurations on Linux VPS servers. It manages SSH users, generates keys, and creates client configurations. It does NOT install sing-box - only detects and manages configurations.

## Documentation Guide

### Always Read First: AGENTS.md

**AGENTS.md** is the primary reference for AI agents. Read it for:
- Project overview and tech stack
- Key design decisions and rationale
- Code style guidelines (including NO EMOJIS rule)
- Common implementation patterns
- Versioning rules
- Quick links to all other documentation

### When Working On Specific Areas

| Working On | Read These Docs |
|------------|----------------|
| **Understanding system design** | `docs/architecture.md` - Module structure, data flow, component interaction |
| **Adding/modifying features** | `docs/features.md` - Feature specifications and requirements |
| **Database changes** | `docs/database.md` - Complete SQLite schema, tables, indexes, relationships |
| **TUI screens/navigation** | `docs/ui-screens.md` - Screen mockups, navigation flow, user interactions |
| **Config generation** | `docs/configuration.md` - sing-box config templates and generation logic |
| **Traffic routing rules** | `docs/routing.md` - Routing presets, custom rules, geosite integration |
| **Logging system** | `docs/logging.md` - Session-based logging, log levels, audit trails |
| **Development phases** | `docs/phases.md` - Development roadmap, WBS, test criteria |
| **Setting up dev environment** | `docs/development.md` - Local setup, testing, contribution guidelines |
| **Installation/deployment** | `docs/installation.md` - Installation methods, scripts, requirements |

### Lab Environment (lab/)

The `lab/` directory contains a complete local development environment for testing sbconfig in an isolated container. It provides a production-like Linux environment with systemd, SSH, and all dependencies.

**When to use lab:**
- Testing system-level features (user creation, SSH config, systemd services)
- Validating changes before VPS deployment
- Testing installation and setup wizard
- Reproducing production-like behavior locally
- Integration testing that requires root privileges

**Lab scripts (use these instead of ad hoc Docker commands):**
```bash
lab/scripts/lab-build.sh    # Build sbconfig for lab environment
lab/scripts/lab-deploy.sh   # Deploy to lab container
lab/scripts/lab-test.sh     # Run integration tests in lab
```

**Lab contents:**
- `lab/Dockerfile` - Container definition with systemd support
- `lab/scripts/` - Build, deploy, and test automation
- `lab/configs/` - Test configurations and sample data

See `docs/lab-setup.md` for complete lab environment documentation and troubleshooting.

### Documentation Reading Strategy

1. **First time in codebase?** → Read AGENTS.md completely
2. **Specific task?** → Check table above for relevant docs
3. **Adding new feature?** → Read docs/features.md + docs/architecture.md + specific area docs
4. **Fixing bug?** → Read relevant area docs, check docs/logging.md for debugging
5. **Testing changes?** → Use lab/ environment per docs/lab-setup.md

### Common Objectives & Required Reading

**Objective: Add a new TUI screen**
1. Read: `docs/ui-screens.md` (navigation patterns)
2. Read: `docs/architecture.md` (UI layer structure)
3. Reference: AGENTS.md (TUI patterns section)

**Objective: Modify database schema**
1. Read: `docs/database.md` (complete schema)
2. Read: AGENTS.md (database operations guidelines)
3. Update: `src/db/schema.sql`, `src/db/migrations.rs`, `src/db/queries.rs`

**Objective: Change config generation**
1. Read: `docs/configuration.md` (config templates)
2. Read: `docs/routing.md` (if routing changes needed)
3. Reference: `docs/features.md` (config generation requirements)

**Objective: Add new sing-box integration**
1. Read: `docs/architecture.md` (sing-box module)
2. Read: `docs/configuration.md` (config format)
3. Remember: Detection only, no installation

**Objective: Setup local testing**
1. Read: `docs/lab-setup.md` (complete lab setup)
2. Read: `docs/development.md` (dev environment)
3. Use: `lab/scripts/` for all lab operations

**Objective: Understand traffic flow**
1. Read: `docs/architecture.md` (system architecture diagram)
2. Read: `docs/routing.md` (routing presets and rules)
3. Read: `docs/configuration.md` (how configs use routing)

## Build Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build (optimized for size)
cargo test                     # Run all tests
cargo test db::                # Test specific module
RUST_LOG=debug cargo test      # Run tests with logging

# Cross-compile for Linux
cross build --release --target x86_64-unknown-linux-musl
cross build --release --target aarch64-unknown-linux-musl
```

## Lab Testing

Use the existing lab scripts for testing in a containerized environment:
```bash
lab/scripts/lab-build.sh       # Build for lab
lab/scripts/lab-deploy.sh      # Deploy to lab container
lab/scripts/lab-test.sh        # Run lab tests
```

Do not use ad hoc Docker images unless explicitly requested.

## Architecture

### Module Structure

- **src/ui/** - TUI layer using ratatui/crossterm
  - `app.rs` - Main application state machine (largest file - handles all screens, navigation, forms)
  - `terminal.rs` - Terminal init/teardown
- **src/db/** - SQLite database layer
  - `database.rs` - Connection management
  - `schema.sql` - Table definitions (users, settings, configs, traffic_usage, etc.)
  - `queries.rs` - All SQL operations
  - `encryption.rs` - AES-GCM encryption for private keys
- **src/ssh/** - SSH operations
  - `keys.rs` - ED25519/RSA key generation
  - `users.rs` - Linux system user CRUD (creates users with /sbin/nologin shell)
  - `ports.rs` - sshd_config port management
  - `service.rs` - sshd service control
- **src/singbox/** - sing-box integration
  - `detect.rs` - Installation detection only
  - `config.rs` - Client config generation with routing presets
  - `service.rs` - systemd service control
- **src/utils/** - System command wrappers
- **src/error.rs** - Custom error types using thiserror

### Key Design Decisions

1. **Custom SSH Port** - Proxy users connect on a dedicated port (e.g., 7344), NOT port 22. Port 22 is reserved for admin access.

2. **No Shell Access** - Proxy users are created with `/sbin/nologin` shell. SSH tunneling still works.

3. **Operating Modes** - Development mode uses localhost; Production mode uses domain/public IP.

4. **Embedded Private Keys** - Client configs contain embedded keys for easy mobile import.

5. **Detection Only** - Does NOT install sing-box; shows installation instructions if missing.

### Routing Presets

The config generator supports routing presets: Default (all traffic proxied), Iran Direct, Iran Block, China Direct - using external geosite rule sets.

## Code Guidelines

- Use `anyhow` for application error handling, `thiserror` for library-style errors
- **NO EMOJIS** anywhere - code, docs, UI, commit messages
- All SQL queries go in `src/db/queries.rs` using prepared statements
- TUI screens are modules in `src/ui/` with navigation via screen stack in App
- Always bump `Cargo.toml` version when shipping user-facing changes
- Private keys are stored encrypted (AES-GCM) in the SQLite database

## Complete Documentation Reference

All project documentation lives in these files:

| File | Purpose |
|------|---------|
| **AGENTS.md** | Primary AI agent reference - start here |
| **README.md** | User-facing project overview and quick start |
| **docs/architecture.md** | System design, modules, data flow |
| **docs/features.md** | Complete feature specifications |
| **docs/database.md** | SQLite schema, tables, indexes, relationships |
| **docs/ui-screens.md** | TUI screen mockups and navigation flow |
| **docs/configuration.md** | sing-box config generation logic |
| **docs/routing.md** | Traffic routing presets and custom rules |
| **docs/logging.md** | Session-based logging system |
| **docs/phases.md** | Development roadmap and test criteria |
| **docs/development.md** | Dev setup and contribution guidelines |
| **docs/installation.md** | Installation methods and deployment |
| **docs/lab-setup.md** | Local testing environment setup |

When in doubt, start with AGENTS.md and follow links to specific documentation as needed.
