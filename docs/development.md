# Development

> Development setup and guidelines for contributing to sbconfig.

## Prerequisites

### Required Tools

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75+ | Core language |
| Cargo | Latest | Package manager |
| Git | Latest | Version control |
| cross | Latest | Cross-compilation (optional) |

### Install Rust

```bash
# Install rustup (Rust toolchain manager)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

### Install Development Tools

```bash
# Install cross for cross-compilation
cargo install cross

# Install cargo-watch for auto-rebuild
cargo install cargo-watch

# Install cargo-audit for security checks
cargo install cargo-audit
```

## Project Setup

### Clone Repository

```bash
git clone https://github.com/YOUR_USERNAME/sbconfig.git
cd sbconfig
```

### Build

```bash
# Debug build (fast compilation, larger binary)
cargo build

# Release build (optimized, smaller binary)
cargo build --release
```

### Run

```bash
# Run debug build
sudo cargo run

# Run release build
sudo cargo run --release

# Run with arguments
sudo cargo run -- --help
```

### Test

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_user_creation

# Run tests for a module
cargo test db::
```

## Project Structure

```
sbconfig/
├── Cargo.toml               # Dependencies and metadata
├── Cargo.lock               # Locked dependency versions
├── src/
│   ├── main.rs              # Entry point
│   ├── app.rs               # App state management
│   ├── ui/                  # TUI components
│   │   ├── mod.rs
│   │   ├── home.rs
│   │   ├── setup.rs
│   │   ├── users.rs
│   │   ├── configs.rs
│   │   ├── settings.rs
│   │   ├── logs.rs
│   │   └── components.rs
│   ├── db/                  # Database layer
│   │   ├── mod.rs
│   │   ├── schema.rs
│   │   └── queries.rs
│   ├── ssh/                 # SSH operations
│   │   ├── mod.rs
│   │   ├── keys.rs
│   │   └── users.rs
│   ├── singbox/             # sing-box operations
│   │   ├── mod.rs
│   │   ├── detect.rs
│   │   ├── config.rs
│   │   └── service.rs
│   └── utils/               # Utilities
│       ├── mod.rs
│       └── system.rs
├── tests/                   # Integration tests
│   ├── db_tests.rs
│   ├── ssh_tests.rs
│   └── config_tests.rs
└── docs/                    # Documentation
```

## Dependencies (Cargo.toml)

```toml
[package]
name = "sbconfig"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your@email.com>"]
description = "TUI tool for managing sing-box SSH configurations"
license = "MIT"
repository = "https://github.com/YOUR_USERNAME/sbconfig"
keywords = ["sing-box", "ssh", "proxy", "tui"]
categories = ["command-line-utilities"]

[dependencies]
# TUI
ratatui = "0.28"
crossterm = "0.28"

# Database
rusqlite = { version = "0.32", features = ["bundled"] }

# SSH Key Generation
ssh-key = { version = "0.6", features = ["ed25519", "rsa", "encryption"] }
rand = "0.8"

# Config & Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# QR Code
qrcode = "0.14"

# Encryption
aes-gcm = "0.10"
pbkdf2 = "0.12"
sha2 = "0.10"

# System Operations
which = "6.0"
users = "0.11"
nix = { version = "0.29", features = ["user", "process"] }

# Utilities
anyhow = "1.0"
thiserror = "1.0"
base64 = "0.22"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4.5", features = ["derive"] }
dirs = "5.0"
log = "0.4"
env_logger = "0.11"

[dev-dependencies]
tempfile = "3.10"
assert_cmd = "2.0"
predicates = "3.1"

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit for better optimization
panic = "abort"     # Abort on panic (smaller binary)
strip = true        # Strip symbols
```

## Code Style

### Formatting

```bash
# Format all code
cargo fmt

# Check formatting without changing
cargo fmt -- --check
```

### Linting

```bash
# Run clippy
cargo clippy

# Run clippy with all warnings
cargo clippy -- -W clippy::all

# Fix clippy warnings automatically
cargo clippy --fix
```

### Code Guidelines

1. **Error Handling**: Use `anyhow::Result` in application code
   ```rust
   use anyhow::{Result, Context, bail};
   
   pub fn do_something() -> Result<()> {
       some_operation().context("Failed to do something")?;
       Ok(())
   }
   ```

2. **Logging**: Use the `log` crate
   ```rust
   use log::{info, warn, error, debug};
   
   info!("Starting operation");
   debug!("Debug details: {:?}", data);
   ```

3. **Documentation**: Add doc comments to public items
   ```rust
   /// Creates a new user with the given username.
   ///
   /// # Arguments
   /// * `username` - The username for the new user
   ///
   /// # Returns
   /// The created `User` struct
   ///
   /// # Errors
   /// Returns an error if the user already exists
   pub fn create_user(username: &str) -> Result<User> {
       // ...
   }
   ```

4. **Testing**: Write tests for all public functions
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_create_user() {
           let result = create_user("test_user");
           assert!(result.is_ok());
       }
   }
   ```

## Development Workflow

### Feature Development

1. Create a feature branch
   ```bash
   git checkout -b feature/my-feature
   ```

2. Make changes and test
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```

3. Commit with conventional commits
   ```bash
   git commit -m "feat: add user deletion confirmation"
   git commit -m "fix: handle empty username error"
   git commit -m "docs: update README with new feature"
   ```

4. Push and create PR
   ```bash
   git push -u origin feature/my-feature
   ```

### Testing on Linux

Since sbconfig requires root and Linux-specific features, test on a VM or container:

```bash
# Using Docker
docker run -it --rm -v $(pwd):/app rust:latest bash
cd /app
cargo build
cargo test
```

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Enable specific module logging
RUST_LOG=sbconfig::db=debug cargo run
```

## Cross-Compilation

### Install Targets

```bash
# Linux x86_64
rustup target add x86_64-unknown-linux-musl

# Linux ARM64
rustup target add aarch64-unknown-linux-musl
```

### Build with Cross

```bash
# Install cross
cargo install cross

# Build for x86_64
cross build --release --target x86_64-unknown-linux-musl

# Build for ARM64
cross build --release --target aarch64-unknown-linux-musl
```

### Build Script

```bash
#!/bin/bash
# build-all.sh

set -e

TARGETS=(
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-musl"
)

for target in "${TARGETS[@]}"; do
    echo "Building for $target..."
    cross build --release --target "$target"
    
    # Create tarball
    tar -czf "sbconfig-$target.tar.gz" \
        -C "target/$target/release" sbconfig
done

echo "Build complete!"
```

## GitHub Actions CI/CD

### `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        
      - name: Run tests
        run: cargo test --verbose
        
      - name: Run clippy
        run: cargo clippy -- -D warnings
        
      - name: Check formatting
        run: cargo fmt -- --check

  build:
    needs: test
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - x86_64-unknown-linux-musl
          - aarch64-unknown-linux-musl
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}
          
      - name: Install cross
        run: cargo install cross
        
      - name: Build
        run: cross build --release --target ${{ matrix.target }}
        
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: sbconfig-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/sbconfig
```

### `.github/workflows/release.yml`

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-musl
            name: linux-amd64
          - target: aarch64-unknown-linux-musl
            name: linux-arm64
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        
      - name: Install cross
        run: cargo install cross
        
      - name: Build
        run: cross build --release --target ${{ matrix.target }}
        
      - name: Package
        run: |
          tar -czf sbconfig-${{ matrix.name }}.tar.gz \
            -C target/${{ matrix.target }}/release sbconfig
            
      - name: Upload Release Asset
        uses: softprops/action-gh-release@v1
        with:
          files: sbconfig-${{ matrix.name }}.tar.gz
```

## Adding New Features

### Adding a New Screen

1. Create file `src/ui/new_screen.rs`:
   ```rust
   use ratatui::prelude::*;
   use crate::app::Action;
   
   pub struct NewScreen {
       // Screen state
   }
   
   impl NewScreen {
       pub fn new() -> Self {
           Self {}
       }
       
       pub fn render(&self, frame: &mut Frame, area: Rect) {
           // Render UI
       }
       
       pub fn handle_input(&mut self, key: KeyEvent) -> Option<Action> {
           // Handle keyboard input
           None
       }
   }
   ```

2. Add to `src/ui/mod.rs`:
   ```rust
   mod new_screen;
   pub use new_screen::NewScreen;
   ```

3. Add navigation in `src/app.rs`

### Adding a Database Table

1. Add migration in `src/db/schema.rs`:
   ```rust
   pub const MIGRATION_002: &str = r#"
       CREATE TABLE new_table (
           id INTEGER PRIMARY KEY,
           ...
       );
   "#;
   ```

2. Add queries in `src/db/queries.rs`

3. Add model struct

### Adding a Command-Line Option

1. Update `src/main.rs`:
   ```rust
   use clap::Parser;
   
   #[derive(Parser)]
   #[command(name = "sbconfig")]
   struct Cli {
       #[arg(short, long)]
       verbose: bool,
       
       #[arg(long)]
       new_option: Option<String>,
   }
   ```

## Troubleshooting

### Build Errors

**SQLite linking errors:**
```bash
# Use bundled SQLite
cargo build --features rusqlite/bundled
```

**Cross-compilation errors:**
```bash
# Install musl tools
sudo apt install musl-tools

# Or use cross
cargo install cross
cross build --target x86_64-unknown-linux-musl
```

### Test Failures

**Permission errors:**
```bash
# Tests requiring root should be marked
#[test]
#[ignore] // Requires root
fn test_system_user_creation() {
    // ...
}

# Run ignored tests with sudo
sudo cargo test -- --ignored
```

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [ratatui Documentation](https://ratatui.rs/)
- [rusqlite Documentation](https://docs.rs/rusqlite/)
- [ssh-key Documentation](https://docs.rs/ssh-key/)
- [sing-box Documentation](https://sing-box.sagernet.org/)

## Related Documentation

- [Architecture](./architecture.md) - System design
- [Features](./features.md) - Feature specifications
- [Database](./database.md) - Database schema
- [Routing](./routing.md) - Traffic routing configuration
- [Logging](./logging.md) - Session-based logging system
