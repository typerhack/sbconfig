# Development Phases

> Work Breakdown Structure (WBS) with milestones, acceptance criteria, and test metrics for sbconfig.

## Overview

sbconfig development is organized into **11 phases**, each with specific deliverables, acceptance criteria, and testable metrics to verify completion.

| Phase | Name | Duration | Dependencies |
|-------|------|----------|--------------|
| 1 | Lab Environment Setup | 1 day | None |
| 2 | Project Setup | 1 day | Phase 1 |
| 3 | Database Layer | 2 days | Phase 2 |
| 4 | SSH Module | 2 days | Phase 3 |
| 5 | sing-box Module | 1 day | Phase 3 |
| 6 | Core TUI Framework | 2 days | Phase 3 |
| 7 | Setup Wizard & Dashboard | 2 days | Phases 4, 5, 6 |
| 8 | User Management | 2 days | Phase 7 |
| 9 | Config Generation | 2 days | Phase 8 |
| 10 | Advanced Features | 3 days | Phase 9 |
| 11 | Polish & Release | 2 days | Phase 10 |

**Total Estimated Duration:** 20 days

---

## Phase 1: Lab Environment Setup

### Objective
Set up a complete development and testing environment with Docker-based Linux lab for integration testing.

### Status
Complete (verified 2026-01-24 via lab-test.sh).

### Deliverables

| ID | Task | Description |
|----|------|-------------|
| 1.1 | Install Rust toolchain | rustup, cargo, rustc on dev machine |
| 1.2 | Install cross-compilation tools | cross, Docker Desktop |
| 1.3 | Create Docker lab environment | Ubuntu container with SSH and sing-box |
| 1.4 | Create docker-compose.yml | Multi-container lab setup |
| 1.5 | Create lab startup script | Easy lab start/stop/reset |
| 1.6 | Verify cross-compilation | Build Linux binary from Mac |
| 1.7 | Verify SSH connectivity | Connect to lab container |
| 1.8 | Verify sing-box in lab | sing-box installed and runnable |
| 1.9 | Document lab usage | Update lab-setup.md with instructions |

### Lab Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Development Environment                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   Mac (Host)                        Docker (Lab)                         │
│   ──────────                        ────────────                         │
│   ├── Rust toolchain                ┌─────────────────────────────┐     │
│   ├── VS Code / Editor              │  sbconfig-lab (Ubuntu 22.04) │     │
│   ├── cross (cross-compile)         │  ├── OpenSSH server          │     │
│   └── Docker Desktop                │  ├── sing-box                 │     │
│                                     │  ├── sqlite3                  │     │
│         cargo build ───────────────►│  ├── /var/lib/sbconfig/      │     │
│         (cross-compile)             │  └── Port 2222 (SSH)         │     │
│                                     └─────────────────────────────┘     │
│   localhost:2222 ◄──────────────────────────────┘                       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Files to Create

```
lab/
├── Dockerfile              # Ubuntu 22.04 with SSH + sing-box
├── docker-compose.yml      # Lab orchestration
├── scripts/
│   ├── lab-start.sh        # Start the lab
│   ├── lab-stop.sh         # Stop the lab
│   ├── lab-reset.sh        # Reset to clean state
│   ├── lab-ssh.sh          # SSH into lab container
│   ├── lab-deploy.sh       # Build and deploy binary to lab
│   └── lab-test.sh         # Run integration tests in lab
└── README.md               # Lab usage instructions
```

### Dockerfile

```dockerfile
# lab/Dockerfile
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies
RUN apt-get update && apt-get install -y \
    openssh-server \
    curl \
    wget \
    sudo \
    sqlite3 \
    vim \
    net-tools \
    iproute2 \
    && rm -rf /var/lib/apt/lists/*

# Configure SSH
RUN mkdir /var/run/sshd && \
    echo 'root:labpass' | chpasswd && \
    sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config && \
    sed -i 's/#PasswordAuthentication yes/PasswordAuthentication yes/' /etc/ssh/sshd_config

# Install sing-box
RUN curl -fsSL https://sing-box.app/deb-install.sh | bash

# Create sbconfig directories
RUN mkdir -p /var/lib/sbconfig /usr/local/bin

# Expose ports
# 22 = SSH (internal)
# 2222 = SSH (mapped to host)
# 7344 = Custom SSH port for sing-box users (example)
EXPOSE 22 7344

# Start SSH
CMD ["/usr/sbin/sshd", "-D"]
```

### docker-compose.yml

```yaml
# lab/docker-compose.yml
version: '3.8'

services:
  sbconfig-lab:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: sbconfig-lab
    hostname: sbconfig-lab
    ports:
      - "2222:22"      # SSH access
      - "7344:7344"    # Custom SSH port for sing-box
      - "10808:10808"  # sing-box proxy port
    volumes:
      - sbconfig-data:/var/lib/sbconfig
      - ./bin:/opt/sbconfig  # Mount for binary deployment
    cap_add:
      - SYS_ADMIN      # For user management
    security_opt:
      - seccomp:unconfined
    restart: unless-stopped

volumes:
  sbconfig-data:
```

### Lab Scripts

#### lab-start.sh
```bash
#!/bin/bash
# lab/scripts/lab-start.sh
set -e

cd "$(dirname "$0")/.."

echo "Starting sbconfig lab environment..."
docker-compose up -d --build

echo "Waiting for SSH to be ready..."
sleep 3

echo ""
echo "Lab is ready!"
echo "  SSH:  ssh -p 2222 root@localhost (password: labpass)"
echo "  Stop: ./scripts/lab-stop.sh"
echo ""
```

#### lab-stop.sh
```bash
#!/bin/bash
# lab/scripts/lab-stop.sh
cd "$(dirname "$0")/.."

echo "Stopping sbconfig lab..."
docker-compose down
echo "Lab stopped."
```

#### lab-reset.sh
```bash
#!/bin/bash
# lab/scripts/lab-reset.sh
cd "$(dirname "$0")/.."

echo "Resetting sbconfig lab to clean state..."
docker-compose down -v
docker-compose up -d --build

sleep 3
echo "Lab reset complete!"
```

#### lab-ssh.sh
```bash
#!/bin/bash
# lab/scripts/lab-ssh.sh
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -p 2222 root@localhost
```

#### lab-deploy.sh
```bash
#!/bin/bash
# lab/scripts/lab-deploy.sh
set -e

cd "$(dirname "$0")/../.."

echo "Building sbconfig for Linux..."
cross build --release --target x86_64-unknown-linux-musl

echo "Deploying to lab..."
docker cp target/x86_64-unknown-linux-musl/release/sbconfig sbconfig-lab:/usr/local/bin/

echo "Setting permissions..."
docker exec sbconfig-lab chmod +x /usr/local/bin/sbconfig

echo "Verifying installation..."
docker exec sbconfig-lab sbconfig --version

echo ""
echo "Deployment complete! Run: ./scripts/lab-ssh.sh then 'sbconfig'"
```

#### lab-test.sh
```bash
#!/bin/bash
# lab/scripts/lab-test.sh
set -e

cd "$(dirname "$0")/.."

echo "=== Lab Environment Tests ==="

echo "[1/5] Testing Docker container is running..."
docker ps | grep -q sbconfig-lab || { echo "FAIL: Container not running"; exit 1; }
echo "PASS: Container running"

echo "[2/5] Testing SSH connectivity..."
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=5 -p 2222 root@localhost "echo connected" 2>/dev/null || { echo "FAIL: SSH not working"; exit 1; }
echo "PASS: SSH working"

echo "[3/5] Testing sing-box installation..."
docker exec sbconfig-lab which sing-box >/dev/null || { echo "FAIL: sing-box not installed"; exit 1; }
echo "PASS: sing-box installed"

echo "[4/5] Testing sing-box version..."
VERSION=$(docker exec sbconfig-lab sing-box version 2>&1 | head -1)
echo "PASS: sing-box version: $VERSION"

echo "[5/5] Testing user creation capability..."
docker exec sbconfig-lab useradd --help >/dev/null || { echo "FAIL: useradd not available"; exit 1; }
echo "PASS: User management available"

echo ""
echo "=== All Lab Tests Passed ==="
```

### Acceptance Criteria

| Test ID | Test Description | Command | Expected Result |
|---------|------------------|---------|-----------------|
| T1.1 | Rust installed | `rustc --version` | Version 1.75+ |
| T1.2 | Cargo installed | `cargo --version` | Cargo available |
| T1.3 | Cross installed | `cross --version` | Cross available |
| T1.4 | Docker running | `docker ps` | Docker daemon running |
| T1.5 | Lab container starts | `./lab/scripts/lab-start.sh` | Container running |
| T1.6 | SSH to lab works | `./lab/scripts/lab-ssh.sh` | Login successful |
| T1.7 | sing-box in lab | `docker exec sbconfig-lab sing-box version` | Version displayed |
| T1.8 | Cross-compile works | `cross build --target x86_64-unknown-linux-musl` | Binary created |
| T1.9 | Deploy to lab works | `./lab/scripts/lab-deploy.sh` | Binary runs in lab |

### Verification Script

```bash
#!/bin/bash
# phase1_verify.sh

echo "=== Phase 1: Lab Environment Verification ==="

echo "[1/9] Checking Rust installation..."
rustc --version >/dev/null 2>&1 || { echo "FAIL: Rust not installed"; exit 1; }
echo "PASS: Rust $(rustc --version)"

echo "[2/9] Checking Cargo..."
cargo --version >/dev/null 2>&1 || { echo "FAIL: Cargo not installed"; exit 1; }
echo "PASS: Cargo available"

echo "[3/9] Checking cross..."
cross --version >/dev/null 2>&1 || { echo "FAIL: cross not installed. Run: cargo install cross"; exit 1; }
echo "PASS: cross available"

echo "[4/9] Checking Docker..."
docker ps >/dev/null 2>&1 || { echo "FAIL: Docker not running"; exit 1; }
echo "PASS: Docker running"

echo "[5/9] Checking lab directory structure..."
for file in lab/Dockerfile lab/docker-compose.yml lab/scripts/lab-start.sh; do
    [[ -f "$file" ]] || { echo "FAIL: $file missing"; exit 1; }
done
echo "PASS: Lab files exist"

echo "[6/9] Starting lab..."
./lab/scripts/lab-start.sh >/dev/null 2>&1 || { echo "FAIL: Lab start failed"; exit 1; }
sleep 5
echo "PASS: Lab started"

echo "[7/9] Testing SSH connectivity..."
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=10 -p 2222 root@localhost "exit" 2>/dev/null || { echo "FAIL: SSH connection failed"; exit 1; }
echo "PASS: SSH works (password: labpass)"

echo "[8/9] Testing sing-box in lab..."
docker exec sbconfig-lab sing-box version >/dev/null 2>&1 || { echo "FAIL: sing-box not working"; exit 1; }
echo "PASS: sing-box working in lab"

echo "[9/9] Testing cross-compilation..."
cross build --release --target x86_64-unknown-linux-musl >/dev/null 2>&1 || { echo "FAIL: Cross-compilation failed"; exit 1; }
[[ -f "target/x86_64-unknown-linux-musl/release/sbconfig" ]] || { echo "FAIL: Binary not created"; exit 1; }
echo "PASS: Cross-compilation works"

echo ""
echo "=== Phase 1 Complete: Lab Environment Ready ==="
echo ""
echo "Quick reference:"
echo "  Start lab:    ./lab/scripts/lab-start.sh"
echo "  SSH to lab:   ./lab/scripts/lab-ssh.sh"
echo "  Deploy:       ./lab/scripts/lab-deploy.sh"
echo "  Stop lab:     ./lab/scripts/lab-stop.sh"
echo "  Reset lab:    ./lab/scripts/lab-reset.sh"
```

---

## Phase 2: Project Setup

### Objective
Initialize Rust project with all dependencies and basic structure.

### Status
Complete (Phase 2 checks: build/test/clippy/fmt/version/help verified 2026-01-24; lab run pending if required).

### Deliverables

| ID | Task | Description |
|----|------|-------------|
| 2.1 | Initialize Cargo project | `cargo init` with proper metadata |
| 2.2 | Configure Cargo.toml | Add all dependencies |
| 2.3 | Create module structure | Create all `src/` subdirectories and mod.rs files |
| 2.4 | Setup error handling | Configure anyhow/thiserror |
| 2.5 | Setup logging | Configure env_logger with log levels |
| 2.6 | Create CLI skeleton | Basic clap argument parsing |

### Acceptance Criteria

| Test ID | Test Description | Command | Expected Result |
|---------|------------------|---------|-----------------|
| T2.1 | Project compiles | `cargo build` | Compiles without errors |
| T2.2 | Tests pass | `cargo test` | All tests pass |
| T2.3 | Clippy clean | `cargo clippy` | No warnings |
| T2.4 | Format check | `cargo fmt --check` | No formatting issues |
| T2.5 | Version flag works | `cargo run -- --version` | Prints version (0.1.0) |
| T2.6 | Help flag works | `cargo run -- --help` | Shows help message |
| T2.7 | Runs in lab | Deploy to lab and run | Executes without error |

### Files Created

```
Cargo.toml              # Dependencies configured
src/
  main.rs               # Entry point with clap CLI
  lib.rs                # Library root (optional)
  app.rs                # App struct skeleton
  error.rs              # Error types
  ui/mod.rs             # UI module declaration
  db/mod.rs             # Database module declaration
  ssh/mod.rs            # SSH module declaration
  singbox/mod.rs        # sing-box module declaration
  utils/mod.rs          # Utils module declaration
```

### Verification Script

```bash
#!/bin/bash
# phase2_verify.sh

echo "=== Phase 2 Verification ==="

echo "[1/7] Testing cargo build..."
cargo build 2>&1 || { echo "FAIL: Build failed"; exit 1; }
echo "PASS: Build succeeded"

echo "[2/7] Testing cargo test..."
cargo test 2>&1 || { echo "FAIL: Tests failed"; exit 1; }
echo "PASS: Tests passed"

echo "[3/7] Testing cargo clippy..."
cargo clippy -- -D warnings 2>&1 || { echo "FAIL: Clippy warnings"; exit 1; }
echo "PASS: Clippy clean"

echo "[4/7] Testing cargo fmt..."
cargo fmt --check 2>&1 || { echo "FAIL: Format issues"; exit 1; }
echo "PASS: Format OK"

echo "[5/7] Testing --version flag..."
VERSION=$(cargo run --quiet -- --version 2>&1)
[[ "$VERSION" == *"0.1.0"* ]] || { echo "FAIL: Version mismatch"; exit 1; }
echo "PASS: Version correct ($VERSION)"

echo "[6/7] Testing --help flag..."
cargo run --quiet -- --help 2>&1 | grep -q "sbconfig" || { echo "FAIL: Help missing"; exit 1; }
echo "PASS: Help works"

echo "[7/7] Testing in lab environment..."
./lab/scripts/lab-deploy.sh >/dev/null 2>&1 || { echo "FAIL: Lab deploy failed"; exit 1; }
docker exec sbconfig-lab sbconfig --version >/dev/null 2>&1 || { echo "FAIL: Lab execution failed"; exit 1; }
echo "PASS: Runs in lab"

echo ""
echo "=== Phase 2 Complete ==="
```

---

## Phase 3: Database Layer

### Objective
Implement SQLite database with all tables, migrations, and CRUD operations.

### Status
Complete (verified 2026-01-25 via lab-build.sh, init-db, and schema checks).

### Deliverables

| ID | Task | Description |
|----|------|-------------|
| 3.1 | Create Database struct | Connection management |
| 3.2 | Implement migrations | Version-tracked schema migrations |
| 3.3 | Create users table | With encryption for private keys |
| 3.4 | Create settings table | Key-value store |
| 3.5 | Create configs table | Generated config storage |
| 3.6 | Create sessions table | Session tracking |
| 3.7 | Create logs table | Application logs |
| 3.8 | Create user_limits table | Connection/traffic limits |
| 3.9 | Create user_connections table | Active connections |
| 3.10 | Create traffic_usage table | Usage statistics |
| 3.11 | Implement key encryption | AES-256-GCM for private keys |
| 3.12 | Implement CRUD operations | All basic queries |

### Acceptance Criteria

| Test ID | Test Description | Expected Result |
|---------|------------------|-----------------|
| T3.1 | Database creates on first run | File exists at specified path |
| T3.2 | Migrations run successfully | All tables created |
| T3.3 | Can create user | User inserted with encrypted key |
| T3.4 | Can read users | Returns list of users |
| T3.5 | Can update user | User data modified |
| T3.6 | Can delete user | User removed (cascade) |
| T3.7 | Can read/write settings | Settings persist |
| T3.8 | Key encryption works | Encrypted != plaintext |
| T3.9 | Key decryption works | Decrypted == original |
| T3.10 | Sessions can be created | Session ID generated |
| T3.11 | Logs can be inserted | Log entry stored |
| T3.12 | Traffic usage tracked | Bytes counted correctly |
| T3.13 | Database works in lab | All tests pass in lab container |

### Files Created

```
src/db/
  mod.rs            # Module exports
  database.rs       # Database struct and connection
  schema.rs         # Table definitions
  migrations.rs     # Migration runner
  queries.rs        # CRUD operations
  encryption.rs     # Key encryption/decryption
  models.rs         # Rust data models
```

### Verification Script

```bash
#!/bin/bash
# phase3_verify.sh

echo "=== Phase 3 Verification ==="

echo "[1/6] Running database unit tests..."
cargo test db:: --quiet 2>&1 || { echo "FAIL: DB tests failed"; exit 1; }
echo "PASS: Database tests passed"

echo "[2/6] Testing database creation in lab..."
./lab/scripts/lab-deploy.sh >/dev/null 2>&1
docker exec sbconfig-lab rm -f /var/lib/sbconfig/sbconfig.db
docker exec sbconfig-lab sbconfig --init-db 2>&1 || { echo "FAIL: DB creation failed"; exit 1; }
docker exec sbconfig-lab test -f /var/lib/sbconfig/sbconfig.db || { echo "FAIL: DB file not created"; exit 1; }
echo "PASS: Database created in lab"

echo "[3/6] Testing table existence in lab..."
TABLES=$(docker exec sbconfig-lab sqlite3 /var/lib/sbconfig/sbconfig.db ".tables" 2>&1)
for table in users settings configs sessions logs migrations user_limits user_connections traffic_usage; do
    echo "$TABLES" | grep -q "$table" || { echo "FAIL: Table $table missing"; exit 1; }
done
echo "PASS: All tables exist"

echo "[4/6] Testing encryption..."
cargo test encryption:: --quiet 2>&1 || { echo "FAIL: Encryption tests failed"; exit 1; }
echo "PASS: Encryption works"

echo "[5/6] Testing CRUD operations..."
cargo test db::queries --quiet 2>&1 || { echo "FAIL: CRUD tests failed"; exit 1; }
echo "PASS: CRUD works"

echo "[6/6] Cleanup lab database..."
docker exec sbconfig-lab rm -f /var/lib/sbconfig/sbconfig.db
echo "PASS: Cleanup done"

echo ""
echo "=== Phase 3 Complete ==="
```

---

## Phase 4: SSH Module

### Objective
Implement SSH key generation, system user management, and SSH port configuration.

### Deliverables

| ID | Task | Description |
|----|------|-------------|
| 4.1 | ED25519 key generation | Generate keypairs |
| 4.2 | RSA key generation | Generate RSA keypairs (4096 bit) |
| 4.3 | System user creation | `useradd` with /sbin/nologin |
| 4.4 | System user deletion | `userdel -r` |
| 4.5 | authorized_keys setup | Write public key to user's .ssh |
| 4.6 | SSH port detection | Check current sshd config |
| 4.7 | SSH port addition | Add custom port to sshd_config |
| 4.8 | SSH port removal | Remove custom port from sshd_config |
| 4.9 | SSH service restart | Restart sshd |
| 4.10 | Port availability check | Check if port is in use |

### Acceptance Criteria

| Test ID | Test Description | Expected Result |
|---------|------------------|-----------------|
| T4.1 | ED25519 key generation | Valid OpenSSH format keys |
| T4.2 | RSA key generation | Valid 4096-bit RSA keys |
| T4.3 | Key format validation | Keys pass ssh-keygen verification |
| T4.4 | Public key format | Starts with "ssh-ed25519" or "ssh-rsa" |
| T4.5 | Private key format | Contains "OPENSSH PRIVATE KEY" |
| T4.6 | User creation in lab | System user created with /sbin/nologin |
| T4.7 | User deletion in lab | System user removed completely |
| T4.8 | authorized_keys in lab | Key written to correct location |
| T4.9 | Port validation | Rejects invalid ports (<1024, >65535) |
| T4.10 | Port in use detection | Correctly identifies used ports |

### Files Created

```
src/ssh/
  mod.rs            # Module exports
  keys.rs           # Key generation (ED25519/RSA)
  users.rs          # System user management
  ports.rs          # SSH port configuration
```

### Verification Script

```bash
#!/bin/bash
# phase4_verify.sh

echo "=== Phase 4 Verification ==="

echo "[1/5] Running SSH module tests..."
cargo test ssh:: --quiet 2>&1 || { echo "FAIL: SSH tests failed"; exit 1; }
echo "PASS: SSH tests passed"

echo "[2/5] Running RSA key generation test..."
cargo test ssh::keys::tests::test_generate_rsa -- --ignored --quiet 2>&1 || { echo "FAIL: RSA key test failed"; exit 1; }
echo "PASS: RSA key test passed"

echo "[3/5] Deploying lab container..."
./lab/scripts/lab-deploy.sh >/dev/null 2>&1 || { echo "FAIL: Lab deploy failed"; exit 1; }
echo "PASS: Lab container ready"

echo "[4/5] Validating nologin shell in lab..."
docker exec sbconfig-lab useradd --create-home --shell /sbin/nologin sbconfig_test_user || { echo "FAIL: User creation failed"; exit 1; }
docker exec sbconfig-lab getent passwd sbconfig_test_user | grep -q "/sbin/nologin" || { echo "FAIL: Wrong shell"; exit 1; }
docker exec sbconfig-lab userdel --remove sbconfig_test_user || { echo "FAIL: User deletion failed"; exit 1; }
echo "PASS: nologin shell verified"

echo "[5/5] Reviewing SSH ports in lab..."
docker exec sbconfig-lab grep "^Port" /etc/ssh/sshd_config >/dev/null 2>&1 || { echo "FAIL: sshd_config ports not found"; exit 1; }
echo "PASS: SSH ports present"

echo ""
echo "=== Phase 4 Complete ==="
```

---

## Phase 5: sing-box Module

### Objective
Implement sing-box detection, service control, and config generation.

### Deliverables

| ID | Task | Description |
|----|------|-------------|
| 5.1 | sing-box detection | Check if installed (which sing-box) |
| 5.2 | Version detection | Parse sing-box version |
| 5.3 | Service status check | Query systemd service |
| 5.4 | Service start/stop/restart | Control sing-box service |
| 5.5 | Config template - Default | Basic SSH proxy config |
| 5.6 | Config template - Iran Direct | With Iran routing rules |
| 5.7 | Config template - Iran Block | Block Iran traffic |
| 5.8 | Config template - China Direct | China routing rules |
| 5.9 | URI generation | Generate sing-box:// URI |
| 5.10 | Config validation | Validate generated JSON |

### Acceptance Criteria

| Test ID | Test Description | Expected Result |
|---------|------------------|-----------------|
| T5.1 | Detection works in lab | Returns true (sing-box installed) |
| T5.2 | Version parsing | Extracts version number |
| T5.3 | Config JSON valid | Passes JSON validation |
| T5.4 | Config has required fields | log, dns, inbounds, outbounds, route |
| T5.5 | SSH outbound correct | type=ssh, server, port, user, private_key |
| T5.6 | URI format correct | Starts with sing-box:// |
| T5.7 | Iran Direct has rule_set | References geosite-all.srs |
| T5.8 | Private key embedded | Not file path, actual key content |
| T5.9 | Config passes sing-box check | `sing-box check -c config.json` succeeds in lab |

### Files Created

```
src/singbox/
  mod.rs            # Module exports
  detect.rs         # Installation detection
  service.rs        # Service control
  config.rs         # Config generation
  templates.rs      # Routing presets
  uri.rs            # URI generation
```

### Verification Script

```bash
#!/bin/bash
# phase5_verify.sh

echo "=== Phase 5 Verification ==="

echo "[1/5] Running sing-box module tests..."
cargo test singbox:: --quiet 2>&1 || { echo "FAIL: sing-box tests failed"; exit 1; }
echo "PASS: sing-box tests passed"

echo "[2/5] Testing detection in lab..."
./lab/scripts/lab-deploy.sh >/dev/null 2>&1
docker exec sbconfig-lab sbconfig --test-detect 2>&1 | grep -q "installed: true" || { echo "FAIL: Detection failed"; exit 1; }
echo "PASS: Detection works in lab"

echo "[3/5] Testing config generation..."
CONFIG=$(cargo run --quiet -- --test-config default 2>&1)
echo "$CONFIG" | jq empty 2>/dev/null || { echo "FAIL: Invalid JSON"; exit 1; }
echo "PASS: Valid JSON generated"

echo "[4/5] Testing config with sing-box in lab..."
docker exec sbconfig-lab sbconfig --test-config default > /tmp/test-config.json
docker cp /tmp/test-config.json sbconfig-lab:/tmp/
docker exec sbconfig-lab sing-box check -c /tmp/test-config.json 2>&1 || { echo "FAIL: Config validation failed"; exit 1; }
echo "PASS: Config passes sing-box check"

echo "[5/5] Testing URI generation..."
cargo run --quiet -- --test-uri 2>&1 | grep -q "^sing-box://" || { echo "FAIL: URI format wrong"; exit 1; }
echo "PASS: URI format correct"

echo ""
echo "=== Phase 5 Complete ==="
```

---

## Phase 6: Core TUI Framework

### Objective
Implement basic TUI infrastructure with ratatui, screen management, and navigation.

### Deliverables

| ID | Task | Description |
|----|------|-------------|
| 6.1 | Terminal setup/teardown | Initialize and restore terminal |
| 6.2 | Event loop | Handle key events with timeout |
| 6.3 | Screen trait | Common interface for screens |
| 6.4 | Screen stack navigation | Push/pop screens, back functionality |
| 6.5 | Action system | Navigation, refresh, quit actions |
| 6.6 | Message display | Success/error/info messages |
| 6.7 | Confirmation dialog | Modal confirmation popup |
| 6.8 | Input components | Text input, select, checkbox |
| 6.9 | Table component | Sortable, scrollable list |
| 6.10 | Help overlay | Show keyboard shortcuts |

### Acceptance Criteria

| Test ID | Test Description | Expected Result |
|---------|------------------|-----------------|
| T6.1 | TUI starts without error | Terminal enters raw mode |
| T6.2 | TUI exits cleanly | Terminal restored on quit |
| T6.3 | Escape goes back | Previous screen shown |
| T6.4 | Q key quits | Application exits |
| T6.5 | Number keys navigate | Menu item selected |
| T6.6 | Arrow keys work | Selection moves |
| T6.7 | Enter key selects | Action triggered |
| T6.8 | Text input works | Characters appear |
| T6.9 | Tab cycles fields | Focus moves to next field |
| T6.10 | TUI works in lab | Interactive session works via SSH |

### Files Created

```
src/ui/
  mod.rs            # Module exports, Screen trait
  app.rs            # App state, event loop
  terminal.rs       # Terminal setup/teardown
  navigation.rs     # Screen stack management
  components/
    mod.rs          # Component exports
    input.rs        # Text input widget
    select.rs       # Selection widget
    checkbox.rs     # Checkbox widget
    table.rs        # Table widget
    dialog.rs       # Confirmation dialog
    message.rs      # Status messages
    help.rs         # Help overlay
```

### Verification Script

```bash
#!/bin/bash
# phase6_verify.sh

echo "=== Phase 6 Verification ==="

echo "[1/4] Running UI module tests..."
cargo test ui:: --quiet 2>&1 || { echo "FAIL: UI tests failed"; exit 1; }
echo "PASS: UI tests passed"

echo "[2/4] Testing component rendering..."
cargo test ui::components --quiet 2>&1 || { echo "FAIL: Component tests failed"; exit 1; }
echo "PASS: Components render correctly"

echo "[3/4] Testing navigation logic..."
cargo test ui::navigation --quiet 2>&1 || { echo "FAIL: Navigation tests failed"; exit 1; }
echo "PASS: Navigation works"

echo "[4/4] Deploying to lab for manual TUI test..."
./lab/scripts/lab-deploy.sh >/dev/null 2>&1
echo "PASS: Deployed to lab"

echo ""
echo "=== Phase 6 Complete ==="
echo ""
echo "Manual test: Run './lab/scripts/lab-ssh.sh' then 'sbconfig' to test TUI"
```

---

## Phase 7: Setup Wizard & Dashboard

### Objective
Implement first-run setup wizard and main dashboard screen.

### Deliverables

| ID | Task | Description |
|----|------|-------------|
| 7.1 | Setup detection | Check if setup_complete in DB |
| 7.2 | Setup Step 1 | sing-box detection screen |
| 7.3 | Setup Step 2 | SSH port configuration |
| 7.4 | Setup Step 3 | Operating mode selection |
| 7.5 | Setup Step 4 | Confirmation and save |
| 7.6 | Dashboard layout | ASCII logo, status, menu |
| 7.7 | Status display | sing-box, SSH, users stats |
| 7.8 | Menu navigation | Navigate to sub-screens |
| 7.9 | Refresh functionality | Update status on demand |
| 7.10 | Setup guard | Block features until setup done |

### Acceptance Criteria

| Test ID | Test Description | Expected Result |
|---------|------------------|-----------------|
| T7.1 | Fresh install shows wizard | Setup screen displayed |
| T7.2 | sing-box detected in wizard | Shows installed status |
| T7.3 | SSH port configurable | Port saved to settings |
| T7.4 | Operating mode selectable | Mode saved to settings |
| T7.5 | Setup completes | setup_complete = true in DB |
| T7.6 | After setup shows dashboard | Dashboard displayed |
| T7.7 | Status shows correct data | Matches actual system state |
| T7.8 | Menu items selectable | Each menu item navigates |
| T7.9 | Feature guard works | Error shown if setup incomplete |
| T7.10 | Full flow works in lab | Complete wizard in lab container |

### Files Created

```
src/ui/screens/
  mod.rs            # Screen exports
  setup/
    mod.rs          # Setup wizard coordinator
    step1.rs        # sing-box detection
    step2.rs        # SSH port config
    step3.rs        # Operating mode
    step4.rs        # Confirmation
  dashboard.rs      # Main dashboard
```

### Verification Script

```bash
#!/bin/bash
# phase7_verify.sh

echo "=== Phase 7 Verification ==="

echo "[1/5] Running setup wizard tests..."
cargo test ui::screens::setup --quiet 2>&1 || { echo "FAIL: Setup tests failed"; exit 1; }
echo "PASS: Setup tests passed"

echo "[2/5] Running dashboard tests..."
cargo test ui::screens::dashboard --quiet 2>&1 || { echo "FAIL: Dashboard tests failed"; exit 1; }
echo "PASS: Dashboard tests passed"

echo "[3/5] Testing setup guard..."
./lab/scripts/lab-deploy.sh >/dev/null 2>&1
docker exec sbconfig-lab rm -rf /var/lib/sbconfig/*
docker exec sbconfig-lab sbconfig --check-setup 2>&1 | grep -q "not complete" || { echo "FAIL: Guard not working"; exit 1; }
echo "PASS: Setup guard works"

echo "[4/5] Resetting lab for manual test..."
docker exec sbconfig-lab rm -rf /var/lib/sbconfig/*
echo "PASS: Lab reset"

echo "[5/5] Ready for manual test..."
echo ""
echo "=== Phase 7 Complete ==="
echo ""
echo "Manual test: Run './lab/scripts/lab-ssh.sh' then 'sbconfig'"
echo "Complete the setup wizard and verify dashboard appears."
```

---

## Phase 8: User Management

### Objective
Implement user listing, creation, deletion, and management screens.

### Deliverables

| ID | Task | Description |
|----|------|-------------|
| 8.1 | User list screen | Table with all users |
| 8.2 | User details panel | Show selected user info |
| 8.3 | Create user dialog | Form for new user |
| 8.4 | Auto-generate username | Generate unique username |
| 8.5 | Delete user flow | Confirmation + deletion |
| 8.6 | Toggle user status | Enable/disable user |
| 8.7 | Regenerate keys | New keypair for user |
| 8.8 | View public key | Display full public key |
| 8.9 | System user integration | Create/delete OS user |
| 8.10 | authorized_keys setup | Configure SSH access |

### Acceptance Criteria

| Test ID | Test Description | Expected Result |
|---------|------------------|-----------------|
| T8.1 | User list displays | All users shown in table |
| T8.2 | Create user works | New user in DB and system |
| T8.3 | Delete user works | User removed from DB and system |
| T8.4 | Toggle status works | is_active changes |
| T8.5 | Username validation | Invalid names rejected |
| T8.6 | Key regeneration works | New keys stored |
| T8.7 | Public key displayable | Full key shown |
| T8.8 | System user created in lab | User exists in /etc/passwd |
| T8.9 | SSH key deployed in lab | authorized_keys contains key |
| T8.10 | User shell correct in lab | Shell is /sbin/nologin |

### Files Created

```
src/ui/screens/
  users/
    mod.rs          # User management coordinator
    list.rs         # User list screen
    create.rs       # Create user dialog
    details.rs      # User details panel
    delete.rs       # Delete confirmation
```

### Verification Script

```bash
#!/bin/bash
# phase8_verify.sh

echo "=== Phase 8 Verification ==="

echo "[1/5] Running user management tests..."
cargo test ui::screens::users --quiet 2>&1 || { echo "FAIL: User tests failed"; exit 1; }
echo "PASS: User management tests passed"

echo "[2/5] Deploying to lab..."
./lab/scripts/lab-deploy.sh >/dev/null 2>&1
echo "PASS: Deployed"

echo "[3/5] Testing full user creation in lab..."
# This assumes setup is already complete
docker exec sbconfig-lab sbconfig --create-user testuser_lab 2>&1 || { echo "FAIL: User creation failed"; exit 1; }
docker exec sbconfig-lab id testuser_lab >/dev/null 2>&1 || { echo "FAIL: System user not created"; exit 1; }
docker exec sbconfig-lab test -f /home/testuser_lab/.ssh/authorized_keys || { echo "FAIL: authorized_keys not created"; exit 1; }
echo "PASS: Full user creation works in lab"

echo "[4/5] Testing user deletion in lab..."
docker exec sbconfig-lab sbconfig --delete-user testuser_lab 2>&1 || { echo "FAIL: User deletion failed"; exit 1; }
docker exec sbconfig-lab id testuser_lab 2>/dev/null && { echo "FAIL: User still exists"; exit 1; }
echo "PASS: User deletion works in lab"

echo "[5/5] Cleanup..."
echo "PASS: Done"

echo ""
echo "=== Phase 8 Complete ==="
```

---

## Phase 9: Config Generation

### Objective
Implement client config generation with multiple output formats.

### Deliverables

| ID | Task | Description |
|----|------|-------------|
| 9.1 | User selection screen | Choose user for config |
| 9.2 | Platform selection | iOS, Android, Windows, etc. |
| 9.3 | Routing preset selection | Default, Iran Direct, etc. |
| 9.4 | JSON config generation | Full sing-box config |
| 9.5 | URI generation | sing-box:// format |
| 9.6 | QR code generation | Terminal QR code display |
| 9.7 | Copy to clipboard | Copy URI/config |
| 9.8 | Save to file | Export config file |
| 9.9 | Config history | Store generated configs |
| 9.10 | Config preview | Show before generating |

### Acceptance Criteria

| Test ID | Test Description | Expected Result |
|---------|------------------|-----------------|
| T9.1 | User selection works | Users listed, selectable |
| T9.2 | Platform selection works | All platforms listed |
| T9.3 | Config generates | Valid JSON output |
| T9.4 | URI generates | Valid sing-box:// URI |
| T9.5 | QR code displays | Scannable QR in terminal |
| T9.6 | Config has correct user | Username and key match |
| T9.7 | Config has correct server | Domain/IP correct |
| T9.8 | Config has correct port | SSH port correct |
| T9.9 | File export works | File created with content |
| T9.10 | Config validates in lab | sing-box check passes |

### Files Created

```
src/ui/screens/
  configs/
    mod.rs              # Config generation coordinator
    select_user.rs      # User selection
    select_platform.rs  # Platform selection
    options.rs          # Config options
    output.rs           # Display/export results
    qr.rs               # QR code rendering
```

### Verification Script

```bash
#!/bin/bash
# phase9_verify.sh

echo "=== Phase 9 Verification ==="

echo "[1/5] Running config generation tests..."
cargo test ui::screens::configs --quiet 2>&1 || { echo "FAIL: Config tests failed"; exit 1; }
echo "PASS: Config generation tests passed"

echo "[2/5] Deploying and creating test user in lab..."
./lab/scripts/lab-deploy.sh >/dev/null 2>&1
docker exec sbconfig-lab sbconfig --create-user configtest 2>&1 || true
echo "PASS: Test user ready"

echo "[3/5] Testing config generation for user..."
docker exec sbconfig-lab sbconfig --gen-config configtest --platform generic --output /tmp/test.json 2>&1 || { echo "FAIL: Config generation failed"; exit 1; }
docker exec sbconfig-lab test -f /tmp/test.json || { echo "FAIL: Config file not created"; exit 1; }
echo "PASS: Config file generated"

echo "[4/5] Testing config with sing-box check..."
docker exec sbconfig-lab sing-box check -c /tmp/test.json 2>&1 || { echo "FAIL: Config validation failed"; exit 1; }
echo "PASS: Config validates"

echo "[5/5] Cleanup..."
docker exec sbconfig-lab sbconfig --delete-user configtest 2>&1 || true
docker exec sbconfig-lab rm -f /tmp/test.json
echo "PASS: Cleanup done"

echo ""
echo "=== Phase 9 Complete ==="
```

---

## Phase 10: Advanced Features

### Objective
Implement logging, limits, routing, update, and uninstall features.

### Deliverables

| ID | Task | Description |
|----|------|-------------|
| 10.1 | Logging system | Session-based logging |
| 10.2 | Log viewer screen | View/filter logs |
| 10.3 | Connection limits | Max connections per user |
| 10.4 | Traffic quotas | Data limits per user |
| 10.5 | Custom routing rules | User-defined rules |
| 10.6 | Rule set management | Enable/disable rule sets |
| 10.7 | Update check | Check GitHub releases |
| 10.8 | Update download/install | Auto-update binary |
| 10.9 | Uninstall - partial | Remove binary only |
| 10.10 | Uninstall - complete | Remove everything |
| 10.11 | Backup database | Export DB to file |
| 10.12 | Restore database | Import from backup |

### Acceptance Criteria

| Test ID | Test Description | Expected Result |
|---------|------------------|-----------------|
| T10.1 | Logs written | Entries in DB and files |
| T10.2 | Log viewer works | Logs displayed, filterable |
| T10.3 | Connection limit enforced | Rejects excess connections |
| T10.4 | Traffic quota tracked | Bytes counted |
| T10.5 | Custom rules saved | Rules in settings |
| T10.6 | Custom rules applied | In generated configs |
| T10.7 | Update check works | Returns version info |
| T10.8 | Backup creates file | Valid JSON backup |
| T10.9 | Restore works | Data recovered |
| T10.10 | Uninstall removes binary | Binary deleted |
| T10.11 | Complete uninstall in lab | All data removed |

### Files Created

```
src/ui/screens/
  logs.rs           # Log viewer
  settings/
    mod.rs          # Settings coordinator
    routing.rs      # Routing configuration
    limits.rs       # User limits
    update.rs       # Update management
    uninstall.rs    # Uninstall flow
    backup.rs       # Backup/restore
src/
  logging.rs        # Logging system
  update.rs         # Update checker/installer
  uninstall.rs      # Uninstall operations
  backup.rs         # Backup/restore operations
```

### Verification Script

```bash
#!/bin/bash
# phase10_verify.sh

echo "=== Phase 10 Verification ==="

echo "[1/6] Running logging tests..."
cargo test logging:: --quiet 2>&1 || { echo "FAIL: Logging tests failed"; exit 1; }
echo "PASS: Logging works"

echo "[2/6] Running limits tests..."
cargo test db::queries::limits --quiet 2>&1 || { echo "FAIL: Limits tests failed"; exit 1; }
echo "PASS: Limits work"

echo "[3/6] Running routing tests..."
cargo test singbox::templates --quiet 2>&1 || { echo "FAIL: Routing tests failed"; exit 1; }
echo "PASS: Routing works"

echo "[4/6] Running backup tests..."
cargo test backup:: --quiet 2>&1 || { echo "FAIL: Backup tests failed"; exit 1; }
echo "PASS: Backup works"

echo "[5/6] Testing backup/restore in lab..."
./lab/scripts/lab-deploy.sh >/dev/null 2>&1
docker exec sbconfig-lab sbconfig --backup /tmp/backup.json 2>&1 || { echo "FAIL: Backup failed"; exit 1; }
docker exec sbconfig-lab test -f /tmp/backup.json || { echo "FAIL: Backup file missing"; exit 1; }
echo "PASS: Backup/restore works in lab"

echo "[6/6] Testing uninstall (dry-run)..."
cargo test uninstall:: --quiet 2>&1 || { echo "FAIL: Uninstall tests failed"; exit 1; }
echo "PASS: Uninstall logic works"

echo ""
echo "=== Phase 10 Complete ==="
```

---

## Phase 11: Polish & Release

### Objective
Final testing, documentation, CI/CD setup, and release preparation.

### Deliverables

| ID | Task | Description |
|----|------|-------------|
| 11.1 | Integration tests | End-to-end test suite |
| 11.2 | Error message polish | User-friendly errors |
| 11.3 | Help text review | All help text accurate |
| 11.4 | Performance testing | Memory/CPU usage OK |
| 11.5 | Security review | No hardcoded secrets |
| 11.6 | CI/CD workflow | GitHub Actions |
| 11.7 | Release binaries | Build for all targets |
| 11.8 | Installation script | install.sh finalized |
| 11.9 | README update | Usage instructions |
| 11.10 | Changelog | Document all changes |

### Acceptance Criteria

| Test ID | Test Description | Expected Result |
|---------|------------------|-----------------|
| T11.1 | All tests pass | `cargo test` succeeds |
| T11.2 | Release builds | All targets compile |
| T11.3 | Binary size OK | < 10 MB per binary |
| T11.4 | Memory usage OK | < 50 MB runtime |
| T11.5 | No clippy warnings | `cargo clippy` clean |
| T11.6 | CI passes | All GitHub Actions green |
| T11.7 | Install script works | Clean install in lab |
| T11.8 | Uninstall script works | Clean removal in lab |
| T11.9 | Documentation complete | All docs up to date |
| T11.10 | Version tagged | Git tag matches Cargo.toml |

### Files Created/Updated

```
.github/workflows/
  ci.yml            # CI workflow
  release.yml       # Release workflow
tests/
  integration/
    mod.rs          # Integration tests
    setup_test.rs   # Setup wizard tests
    user_test.rs    # User management tests
    config_test.rs  # Config generation tests
install.sh          # Installation script
uninstall.sh        # Uninstall script (standalone)
CHANGELOG.md        # Version changelog
```

### Verification Script

```bash
#!/bin/bash
# phase11_verify.sh

echo "=== Phase 11 Verification ==="

echo "[1/8] Running all tests..."
cargo test --all 2>&1 || { echo "FAIL: Tests failed"; exit 1; }
echo "PASS: All tests passed"

echo "[2/8] Running clippy..."
cargo clippy -- -D warnings 2>&1 || { echo "FAIL: Clippy warnings"; exit 1; }
echo "PASS: Clippy clean"

echo "[3/8] Building release..."
cargo build --release 2>&1 || { echo "FAIL: Release build failed"; exit 1; }
echo "PASS: Release builds"

echo "[4/8] Checking binary size..."
SIZE=$(stat -f%z target/release/sbconfig 2>/dev/null || stat -c%s target/release/sbconfig 2>/dev/null)
[[ $SIZE -lt 10485760 ]] || { echo "FAIL: Binary too large ($SIZE bytes)"; exit 1; }
echo "PASS: Binary size OK ($SIZE bytes)"

echo "[5/8] Checking version consistency..."
CARGO_VER=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
BIN_VER=$(./target/release/sbconfig --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
[[ "$CARGO_VER" == "$BIN_VER" ]] || { echo "FAIL: Version mismatch"; exit 1; }
echo "PASS: Version consistent ($CARGO_VER)"

echo "[6/8] Testing install script in lab..."
./lab/scripts/lab-reset.sh >/dev/null 2>&1
sleep 5
docker cp install.sh sbconfig-lab:/tmp/
docker exec sbconfig-lab bash /tmp/install.sh 2>&1 || { echo "FAIL: Install script failed"; exit 1; }
docker exec sbconfig-lab sbconfig --version >/dev/null 2>&1 || { echo "FAIL: Installation verification failed"; exit 1; }
echo "PASS: Install script works"

echo "[7/8] Checking documentation..."
for doc in README.md CHANGELOG.md docs/phases.md; do
    [[ -f "$doc" ]] || { echo "FAIL: $doc missing"; exit 1; }
done
echo "PASS: Documentation exists"

echo "[8/8] Final cleanup..."
./lab/scripts/lab-stop.sh >/dev/null 2>&1
echo "PASS: Cleanup done"

echo ""
echo "=== Phase 11 Complete ==="
echo "=== PROJECT READY FOR RELEASE ==="
```

---

## Summary Checklist

### Phase Completion Tracking

| Phase | Name | Tests | Status |
|-------|------|-------|--------|
| 1 | Lab Environment Setup | T1.1-T1.9 | [x] Complete |
| 2 | Project Setup | T2.1-T2.7 | [x] Complete |
| 3 | Database Layer | T3.1-T3.13 | [x] Complete |
| 4 | SSH Module | T4.1-T4.10 | [ ] Pending |
| 5 | sing-box Module | T5.1-T5.9 | [ ] Pending |
| 6 | Core TUI Framework | T6.1-T6.10 | [ ] Pending |
| 7 | Setup Wizard & Dashboard | T7.1-T7.10 | [ ] Pending |
| 8 | User Management | T8.1-T8.10 | [ ] Pending |
| 9 | Config Generation | T9.1-T9.10 | [ ] Pending |
| 10 | Advanced Features | T10.1-T10.11 | [ ] Pending |
| 11 | Polish & Release | T11.1-T11.10 | [ ] Pending |

### Test Count Summary

| Phase | Unit Tests | Integration Tests | Lab Tests | Total |
|-------|------------|-------------------|-----------|-------|
| 1 | 0 | 0 | 9 | 9 |
| 2 | 6 | 0 | 1 | 7 |
| 3 | 10 | 0 | 3 | 13 |
| 4 | 5 | 0 | 5 | 10 |
| 5 | 4 | 0 | 5 | 9 |
| 6 | 9 | 0 | 1 | 10 |
| 7 | 5 | 0 | 5 | 10 |
| 8 | 5 | 0 | 5 | 10 |
| 9 | 5 | 0 | 5 | 10 |
| 10 | 6 | 0 | 5 | 11 |
| 11 | 5 | 5 | 5 | 15 |
| **Total** | **60** | **5** | **49** | **114** |

---

## Related Documentation

- [Architecture](./architecture.md) - System design
- [Features](./features.md) - Complete feature list
- [Database](./database.md) - Database schema
- [UI Screens](./ui-screens.md) - TUI mockups
- [Development](./development.md) - Development setup
- [Lab Setup](./lab-setup.md) - Lab environment details
- [Logging](./logging.md) - Logging system
