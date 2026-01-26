# Architecture

> System architecture and module structure for sbconfig.

## Overview

sbconfig is designed as a single-binary TUI application that runs directly on a Linux VPS. It manages SSH users and generates client configurations for sing-box.

> **Important Design Decision:** sbconfig does NOT install sing-box. It only detects if sing-box is installed and guides users to install manually. This keeps the tool focused on user/config management.

## Operating Modes

sbconfig supports two operating modes:

### Development Mode
- Server address: `localhost` or `127.0.0.1`
- Used for local testing and development
- Configs generated with localhost address
- No domain validation required

### Production Mode
- Server address: Domain name or public IP
- Used on actual VPS deployment
- Configs generated with real server address
- Domain/IP validation required

```rust
pub enum OperatingMode {
    Development,  // Uses localhost
    Production,   // Uses domain/IP
}
```

## SSH Port Strategy

> **Key Decision:** sbconfig does NOT use the default SSH port (22). It requires a dedicated custom SSH port for sing-box proxy users.

```
┌─────────────────────────────────────────────────────────────────┐
│                     SSH Port Architecture                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Port 22 (Default SSH)     Port 7344 (Example Custom)           │
│  ├── Admin access          ├── sing-box proxy users             │
│  ├── System maintenance    ├── user_alpha                       │
│  └── NOT for sing-box      ├── user_beta                        │
│                            └── user_gamma                        │
│                                                                  │
│  sbconfig manages the custom port only, never port 22           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### First-Run Port Setup
1. Ask user: "Do you have a custom SSH port for sing-box?"
2. If YES → Enter existing port number
3. If NO → Auto-generate random port (10000-60000)
4. Configure sshd or secondary sshd instance
5. Store port in database

## System Architecture

```
┌────────────────────────────────────────────────────────────────────────────┐
│                              sbconfig TUI                                   │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   UI Layer  │  │  App State  │  │  Database   │  │   System    │       │
│  │  (ratatui)  │◄─┤   Manager   │◄─┤  (SQLite)   │  │  Commands   │       │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘       │
│         │                │                │                │               │
│         ▼                ▼                ▼                ▼               │
│  ┌─────────────────────────────────────────────────────────────────┐      │
│  │                        Core Modules                              │      │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │      │
│  │  │   SSH    │  │ sing-box │  │  Config  │  │   QR     │        │      │
│  │  │  Module  │  │  Module  │  │Generator │  │  Module  │        │      │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │      │
│  └─────────────────────────────────────────────────────────────────┘      │
│                                                                             │
└────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│                           Operating System                                  │
├────────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │   OpenSSH    │  │   systemd    │  │  /etc/passwd │  │  Filesystem  │   │
│  │   Server     │  │   Services   │  │  User Mgmt   │  │   Storage    │   │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘   │
└────────────────────────────────────────────────────────────────────────────┘
```

## Module Structure

### Source Code Layout

```
src/
├── main.rs              # Application entry point
├── app.rs               # Central app state and event handling
├── ui/                  # User interface components
│   ├── mod.rs           # UI module exports
│   ├── home.rs          # Dashboard/home screen
│   ├── setup.rs         # First-run setup wizard
│   ├── status.rs        # sing-box status screen (detection only)
│   ├── users.rs         # User management screen
│   ├── configs.rs       # Config generation screen
│   ├── settings.rs      # Server settings screen
│   ├── logs.rs          # Log viewer screen
│   └── components.rs    # Reusable UI widgets
├── db/                  # Database layer
│   ├── mod.rs           # Database module exports
│   ├── schema.rs        # Table definitions and migrations
│   └── queries.rs       # CRUD operations
├── ssh/                 # SSH management
│   ├── mod.rs           # SSH module exports
│   ├── keys.rs          # Key generation (ED25519/RSA)
│   ├── users.rs         # System user CRUD (with /sbin/nologin shell)
│   └── port.rs          # SSH port configuration (sshd_config)
├── singbox/             # sing-box operations
│   ├── mod.rs           # sing-box module exports
│   ├── detect.rs        # Detection only (NOT installation)
│   ├── config.rs        # Config templates and generation
│   └── service.rs       # systemd service management
└── utils/               # Shared utilities
    ├── mod.rs           # Utils module exports
    └── system.rs        # System command wrappers
```

## Component Details

### 1. UI Layer (`src/ui/`)

The UI is built with `ratatui` and `crossterm`. Each screen is a separate module implementing a common interface.

```rust
// Common screen trait
pub trait Screen {
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_input(&mut self, key: KeyEvent) -> Option<Action>;
    fn title(&self) -> &str;
}

// Navigation actions
pub enum Action {
    Navigate(ScreenId),
    Back,
    Quit,
    Refresh,
}
```

**Screens:**
| Screen | File | Purpose |
|--------|------|---------|
| Home | `home.rs` | Dashboard with system status |
| Setup | `setup.rs` | First-run setup wizard (4 steps) |
| Status | `status.rs` | sing-box status (detection only, NOT installer) |
| Users | `users.rs` | User CRUD operations |
| Configs | `configs.rs` | Generate client configs |
| Settings | `settings.rs` | Server configuration |
| Logs | `logs.rs` | View system logs |

### 2. App State (`src/app.rs`)

Central state management using a single `App` struct:

```rust
pub struct App {
    // Operating Mode
    pub mode: OperatingMode,
    pub setup_complete: bool,  // Guards user operations
    
    // Navigation
    pub screen_stack: Vec<ScreenId>,
    pub current_screen: ScreenId,
    
    // Data
    pub db: Database,
    pub settings: Settings,
    
    // UI State
    pub selected_user: Option<i64>,
    pub message: Option<(String, MessageType)>,
    
    // System state cache
    pub singbox_installed: bool,
    pub singbox_version: Option<String>,
    pub ssh_port: Option<u16>,  // None until configured
}

pub enum OperatingMode {
    Development,  // localhost
    Production,   // domain/IP
}
```

#### Setup Guard

User operations are blocked until setup is complete:

```rust
impl App {
    pub fn can_manage_users(&self) -> Result<()> {
        if !self.setup_complete {
            bail!("Setup not complete. Please run the setup wizard first.");
        }
        if !self.singbox_installed {
            bail!("sing-box is not installed. Please install sing-box first.");
        }
        Ok(())
    }
}
```

### 3. Database Layer (`src/db/`)

SQLite database for persistent storage. See [database.md](./database.md) for full schema.

```rust
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self>;
    pub fn migrate(&self) -> Result<()>;
    
    // User operations
    pub fn create_user(&self, user: &NewUser) -> Result<User>;
    pub fn get_users(&self) -> Result<Vec<User>>;
    pub fn delete_user(&self, id: i64) -> Result<()>;
    
    // Settings operations
    pub fn get_setting(&self, key: &str) -> Result<Option<String>>;
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()>;
    
    // Config operations
    pub fn save_config(&self, config: &GeneratedConfig) -> Result<()>;
    pub fn get_configs(&self, user_id: i64) -> Result<Vec<GeneratedConfig>>;
}
```

### 4. SSH Module (`src/ssh/`)

Handles SSH key generation, system user management, and SSH port configuration. User identity includes a username and an optional email for search/notification workflows.

```rust
// Key generation
pub fn generate_ed25519_keypair() -> Result<(String, String)>;  // (private, public)
pub fn generate_rsa_keypair(bits: u32) -> Result<(String, String)>;

// System user management (users have /sbin/nologin shell - no interactive login)
pub fn create_system_user(username: &str) -> Result<()>;
pub fn delete_system_user(username: &str) -> Result<()>;
pub fn setup_ssh_authorized_keys(username: &str, public_key: &str) -> Result<()>;

// SSH port configuration (modifies /etc/ssh/sshd_config)
pub fn add_ssh_port(port: u16) -> Result<()>;
pub fn remove_ssh_port(port: u16) -> Result<()>;
pub fn restart_sshd() -> Result<()>;
pub fn is_port_in_use(port: u16) -> bool;
```

#### User Creation Details

Proxy users are created with restricted shell to prevent interactive login:

```rust
pub fn create_system_user(username: &str) -> Result<()> {
    // Create user with:
    // - Shell: /sbin/nologin (no interactive login, SSH tunneling only)
    // - No password (key-based auth only)
    // - Home directory for .ssh/authorized_keys
    Command::new("useradd")
        .args([
            "--create-home",
            "--shell", "/sbin/nologin",
            "--comment", "sing-box proxy user",
            username,
        ])
        .status()?;
    Ok(())
}
```

#### SSH Port Configuration

sbconfig adds a custom port to `/etc/ssh/sshd_config`:

```rust
pub fn add_ssh_port(port: u16) -> Result<()> {
    // Read current sshd_config
    let config = fs::read_to_string("/etc/ssh/sshd_config")?;
    
    // Check if port already exists
    if config.contains(&format!("Port {}", port)) {
        return Ok(()); // Already configured
    }
    
    // Add new Port line after existing Port lines (or at start if none)
    // Example result in sshd_config:
    // Port 22
    // Port 7344  # Added by sbconfig for sing-box proxy users
    
    // Restart sshd to apply
    restart_sshd()?;
    Ok(())
}
```

#### Connection and Traffic Limits

Per-user limits are stored in `user_limits` (max concurrent connections and traffic quota). Usage is tracked in `user_connections` and `traffic_usage`.

### 5. sing-box Module (`src/singbox/`)

Handles sing-box detection and status checking (NOT installation).

```rust
// Detection (not installation)
pub fn is_installed() -> bool;
pub fn get_version() -> Result<Option<String>>;
pub fn get_install_instructions() -> &'static str;

// Service management (if installed)
pub fn start() -> Result<()>;
pub fn stop() -> Result<()>;
pub fn restart() -> Result<()>;
pub fn status() -> Result<ServiceStatus>;

// Config generation
pub fn generate_client_config(user: &User, settings: &Settings) -> ClientConfig;
pub fn generate_uri(config: &ClientConfig) -> String;
```

> **Note:** The `install()` and `update()` functions are intentionally omitted. Users must install sing-box manually using official instructions.

## Data Flow

### User Creation Flow

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  UI Form │───▶│   App    │───▶│   SSH    │───▶│  System  │───▶│ Database │
│  Input   │    │  Logic   │    │  Module  │    │  Users   │    │  Store   │
└──────────┘    └──────────┘    └──────────┘    └──────────┘    └──────────┘
     │                │                │                │              │
     │  username/email│  validate      │  gen keys      │  useradd     │  INSERT
     │  ──────────▶   │  ──────────▶   │  ──────────▶   │  ──────────▶ │  ──────▶
     │                │                │  setup ssh     │              │
     │                │                │  ──────────▶   │              │
```

### Config Generation Flow

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  Select  │───▶│  Query   │───▶│ Generate │───▶│  Render  │───▶│  Export  │
│  User    │    │   DB     │    │  Config  │    │  QR/URI  │    │  Output  │
└──────────┘    └──────────┘    └──────────┘    └──────────┘    └──────────┘
     │                │                │                │              │
     │  user_id       │  user data     │  JSON config   │  QR code     │  display
     │  platform      │  settings      │  sing-box URI  │  copy URI    │  save file
```

## Error Handling

All modules use `anyhow::Result` for error handling:

```rust
use anyhow::{Result, Context, bail};

pub fn create_user(username: &str) -> Result<User> {
    // Validate
    if username.is_empty() {
        bail!("Username cannot be empty");
    }
    
    // Create with context
    create_system_user(username)
        .context("Failed to create system user")?;
    
    let (private_key, public_key) = generate_ed25519_keypair()
        .context("Failed to generate SSH keys")?;
    
    // ... rest of creation
    Ok(user)
}
```

## Concurrency Model

The TUI runs on a single thread with an event loop:

```rust
fn main() -> Result<()> {
    // Initialize
    let mut terminal = setup_terminal()?;
    let mut app = App::new()?;
    
    // Event loop
    loop {
        // Render
        terminal.draw(|f| app.render(f))?;
        
        // Handle input (with timeout for async updates)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if app.handle_input(key)?.should_quit() {
                    break;
                }
            }
        }
        
        // Background updates (status checks, etc.)
        app.tick()?;
    }
    
    restore_terminal()?;
    Ok(())
}
```

## Security Architecture

### Key Storage

```
┌─────────────────────────────────────────────────────────┐
│                    SQLite Database                       │
│  /var/lib/sbconfig/sbconfig.db                          │
├─────────────────────────────────────────────────────────┤
│  users table:                                            │
│  ├── private_key: AES-256-GCM encrypted                 │
│  └── public_key: plaintext (not sensitive)              │
├─────────────────────────────────────────────────────────┤
│  Encryption key derived from:                            │
│  ├── Machine ID (/etc/machine-id)                       │
│  └── Installation-time random salt                       │
└─────────────────────────────────────────────────────────┘
```

### Permission Model

| Resource | Permission | Reason |
|----------|------------|--------|
| Database file | `600` (root only) | Contains encrypted keys |
| Generated configs | `600` (root only) | Contains private keys |
| Binary | `755` | Executable by all, writes need root |

## File Locations

| Path | Purpose |
|------|---------|
| `/usr/local/bin/sbconfig` | Main binary |
| `/var/lib/sbconfig/` | Data directory |
| `/var/lib/sbconfig/sbconfig.db` | SQLite database |
| `/var/lib/sbconfig/configs/` | Generated config files |
| `/etc/sbconfig/` | Optional: external config |

## Related Documentation

- [Features](./features.md) - Complete feature list
- [Database](./database.md) - Database schema details
- [UI Screens](./ui-screens.md) - Screen mockups
- [Development](./development.md) - Dev setup guide
- [Logging](./logging.md) - Session-based logging system
- [Phases](./phases.md) - Development phases and WBS
