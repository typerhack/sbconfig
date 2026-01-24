# Features

> Complete feature list and specifications for sbconfig.

## Feature Overview

| Category | Features |
|----------|----------|
| sing-box Management | Detection (not install), start/stop, status, version check |
| User Management | Create, list, delete, enable/disable, regenerate keys |
| Config Generation | JSON export, URI generation, QR codes, multi-platform |
| **Routing** | Presets (Iran, China), custom rules, rule sets, block/direct/proxy |
| Server Settings | Custom SSH port, domain/localhost, proxy port configuration |
| Monitoring | Log viewing, service status, connection stats |
| Data Management | SQLite storage, backup, restore |

> **Note:** sbconfig does NOT install sing-box. Users must install it manually before using sbconfig.

---

## 1. Dashboard (Home Screen)

### Status Display
- [ ] sing-box installation status (installed/not installed)
- [ ] sing-box version number
- [ ] sing-box service status (running/stopped)
- [ ] SSH server status (active/inactive)
- [ ] SSH listening port
- [ ] Configured domain/hostname
- [ ] Total number of users
- [ ] Total generated configs count

### Quick Actions
- [ ] Refresh status
- [ ] Quick navigation to all main features

---

## 2. sing-box Management

### Detection (Not Installation)
> **Note:** sbconfig does NOT install sing-box automatically. It only detects if sing-box is installed and guides the user to install it manually if needed.

- [ ] Detect if sing-box is installed (check `which sing-box`)
- [ ] Display installation status on dashboard
- [ ] If not installed, show instructions for manual installation
- [ ] Link to official sing-box installation guide
- [ ] Verify sing-box version compatibility

### Version Check
- [ ] Display installed version
- [ ] Check for available updates (informational only)
- [ ] Show link to update instructions

### Service Control
- [ ] Start sing-box service
- [ ] Stop sing-box service
- [ ] Restart sing-box service
- [ ] Enable/disable auto-start on boot
- [ ] View service status details

### Version Information
- [ ] Display installed version
- [ ] Display latest available version
- [ ] Compare versions

---

## 3. User Management

### User Listing
- [ ] Display all users in table format
- [ ] Show user ID
- [ ] Show username
- [ ] Show creation date
- [ ] Show SSH port assignment
- [ ] Show active/disabled status
- [ ] Show number of generated configs
- [ ] Sort by different columns
- [ ] Filter active/disabled users

### Create User
- [ ] Auto-generate unique username (e.g., `user_abc123`)
- [ ] Allow custom username input
- [ ] Validate username (alphanumeric, no spaces)
- [ ] Check for duplicate usernames
- [ ] Generate ED25519 SSH key pair automatically
- [ ] Create system user on the OS with `/sbin/nologin` shell (no interactive login)
- [ ] Set up SSH authorized_keys for the user
- [ ] Store user data in database
- [ ] Display success confirmation with details
- [ ] **Guard:** Show error if setup wizard not completed

### Delete User
- [ ] Confirm deletion prompt
- [ ] Remove from database
- [ ] Remove system user
- [ ] Remove SSH keys from filesystem
- [ ] Revoke any active sessions (optional)
- [ ] Display deletion confirmation

### Edit User
- [ ] Rename user (with system user rename)
- [ ] Change SSH port assignment
- [ ] Add notes/description

### User Status
- [ ] Enable user (allow connections)
- [ ] Disable user (block connections without deleting)
- [ ] Show enabled/disabled state clearly

### Key Management
- [ ] View public key
- [ ] Regenerate key pair
- [ ] Export private key
- [ ] Key rotation with old key invalidation

---

## 4. Client Config Generation

### User Selection
- [ ] Select from list of active users
- [ ] Show user details before generation
- [ ] Support quick selection by ID

### Platform Selection
- [ ] iOS (sing-box app)
- [ ] Android (sing-box app)
- [ ] Windows
- [ ] macOS
- [ ] Linux
- [ ] Generic (manual configuration)

### Config Options
- [ ] Server address (domain or IP)
- [ ] SSH port
- [ ] Local proxy port (default: 10808)
- [ ] DNS configuration
- [ ] Routing rules (see Routing section below)

### Output Formats

#### JSON Config
- [ ] Generate valid sing-box JSON
- [ ] Pretty-printed for readability
- [ ] Include all necessary fields
- [ ] Embed private key (not file path)

#### sing-box URI
- [ ] Generate `sing-box://` URI format
- [ ] Include all connection parameters
- [ ] URL-encode special characters
- [ ] Copy to clipboard support

#### QR Code
- [ ] Generate QR code from URI
- [ ] Display in terminal using Unicode blocks
- [ ] Appropriate size for scanning
- [ ] High contrast for visibility

### Export Options
- [ ] Display on screen
- [ ] Copy to clipboard
- [ ] Save to file
- [ ] Specify custom filename
- [ ] Default location: `/var/lib/sbconfig/configs/`

### Config History
- [ ] Save generated configs to database
- [ ] View previously generated configs
- [ ] Regenerate from history
- [ ] Delete old configs

---

## 5. Routing Configuration

> Configure how traffic is routed in generated client configs. See [docs/routing.md](./routing.md) for detailed documentation.

### Routing Presets
- [ ] **Default** - Proxy all traffic, direct for private IPs
- [ ] **Iran Direct** - Iranian domains direct, Iranian ads blocked, proxy rest
- [ ] **Iran Block** - Block all traffic to/from Iran
- [ ] **China Direct** - Chinese domains/IPs direct, proxy rest
- [ ] **Custom** - User-defined rules

### Preset Details

| Preset | Description | Rule Sets Used |
|--------|-------------|----------------|
| Default | Basic routing | None |
| Iran Direct | Direct access to Iranian sites | `geosite-all.srs`, `geosite-ads.srs` |
| Iran Block | Block Iranian traffic | `geosite-all.srs`, `geoip-ir.srs` |
| China Direct | Direct access to Chinese sites | `geosite-cn.srs`, `geoip-cn.srs` |

### Custom Rules
- [ ] Add domain-based rules (exact, suffix, keyword, regex)
- [ ] Add IP-based rules (CIDR, private IP)
- [ ] Add port-based rules
- [ ] Add protocol-based rules (dns, tcp, udp)
- [ ] Set rule action: direct, block, or proxy
- [ ] Reorder rules (priority)
- [ ] Delete rules

### Rule Sets (Remote Lists)
- [ ] Enable/disable pre-configured rule sets
- [ ] Iran: ads, all, ir, other, proxy (from bootmortis/sing-geosite)
- [ ] China: geosite-cn, geoip-cn (from SagerNet)
- [ ] Custom URL rule sets
- [ ] Auto-update rule sets (configurable interval)

### Routing UI
- [ ] Select routing preset from list
- [ ] View current routing rules
- [ ] Edit custom rules
- [ ] Preview generated route config
- [ ] Apply routing to all new configs

---

## 6. Server Settings

### SSH Port Configuration
> **Important:** sbconfig does NOT use the system's default SSH port (22). It requires a custom SSH port dedicated to sing-box proxy users.

- [ ] First-run setup: Ask if user already has a custom SSH port configured
- [ ] If yes: Prompt user to enter their existing custom port
- [ ] If no: Auto-generate a random port (range: 10000-60000)
- [ ] Validate port is not in use
- [ ] Validate port is not a well-known port (< 1024)
- [ ] Store configured port in database
- [ ] Display warning if port matches system SSH port (22)
- [ ] Firewall rule suggestions for the custom port

### SSH Port Change
- [ ] Change custom SSH port
- [ ] Update sshd_config with new port (or secondary sshd instance)
- [ ] Restart SSH service after changes
- [ ] Update all generated configs with new port
- [ ] Show firewall update commands

### Domain/Hostname Configuration
> **Development Mode** uses `localhost` or `127.0.0.1`. **Production Mode** requires a domain or public IP.

- [ ] Set server domain name (production)
- [ ] Set server IP address (production)
- [ ] Auto-detect public IP
- [ ] Validate domain format
- [ ] Development mode: Use `localhost` for testing
- [ ] Production mode: Require domain/IP before generating configs
- [ ] Store mode preference in settings

### Proxy Settings
- [ ] Default local proxy port for configs
- [ ] DNS server preferences
- [ ] Default routing mode

### System Settings
- [ ] View system information
- [ ] Check firewall status
- [ ] Network interface info

---

## 7. Log Viewing

### sing-box Logs
- [ ] View sing-box service logs
- [ ] Real-time log streaming
- [ ] Filter by log level (debug, info, warn, error)
- [ ] Search in logs
- [ ] Export logs to file

### SSH Logs
- [ ] View SSH authentication logs
- [ ] Filter by user
- [ ] Show successful/failed attempts
- [ ] Identify connection sources

### Application Logs
- [ ] sbconfig operation logs
- [ ] Error tracking
- [ ] Audit trail for user management

---

## 8. Backup & Restore

### Backup
- [ ] Export entire database
- [ ] Export specific users
- [ ] Export settings only
- [ ] Include/exclude private keys
- [ ] Compressed backup files
- [ ] Timestamped backup names

### Restore
- [ ] Import database backup
- [ ] Merge vs. replace options
- [ ] Validate backup integrity
- [ ] Preview before restore
- [ ] Restore specific users only

---

## 9. UI/UX Features

### Navigation
- [ ] Keyboard navigation (arrow keys)
- [ ] Number key shortcuts
- [ ] Tab navigation between sections
- [ ] Escape to go back
- [ ] `q` to quit
- [ ] Breadcrumb trail

### Visual Elements
- [ ] Color-coded status indicators
- [ ] Progress bars for long operations
- [ ] Loading spinners
- [ ] Success/error messages
- [ ] Confirmation dialogs

### Help System
- [ ] `?` key for context help
- [ ] Keyboard shortcut reference
- [ ] Feature documentation links

### Accessibility
- [ ] High contrast mode
- [ ] No color-only information
- [ ] Screen reader compatible output

---

## 10. Command Line Interface

### Arguments
```
sbconfig [OPTIONS] [COMMAND]

Options:
  -h, --help       Print help
  -V, --version    Print version
  -v, --verbose    Verbose output
  -q, --quiet      Minimal output

Commands:
  tui              Start interactive TUI (default)
  status           Show system status
  users            List all users
  add-user         Add a new user
  del-user         Delete a user
  gen-config       Generate client config
  backup           Create database backup
  restore          Restore from backup
```

### Non-Interactive Mode
- [ ] All features accessible via CLI
- [ ] JSON output option for scripting
- [ ] Exit codes for success/failure
- [ ] Quiet mode for cron jobs

---

## 11. Security Features

### Authentication
- [ ] Require root privileges
- [ ] Validate sudo access

### Data Protection
- [ ] Encrypt private keys in database
- [ ] Secure file permissions
- [ ] Clear sensitive data from memory
- [ ] No logging of private keys

### Input Validation
- [ ] Sanitize all user input
- [ ] Prevent command injection
- [ ] Validate configuration values

---

## 12. First-Run Setup Wizard

> On first launch, sbconfig guides the user through initial configuration.

### Step 1: sing-box Detection
- [ ] Check if sing-box is installed
- [ ] If not installed: Show installation instructions and links
- [ ] If installed: Display version and continue

### Step 2: SSH Port Configuration
- [ ] Ask: "Do you already have a custom SSH port configured for sing-box users?"
- [ ] If yes: Prompt for existing port number
- [ ] If no: Generate random port (10000-60000) and configure it
- [ ] Validate and test the port
- [ ] Show firewall commands if needed

### Step 3: Server Address Configuration
- [ ] Ask: "Are you running in development mode or production?"
- [ ] Development: Set server address to `localhost`
- [ ] Production: Prompt for domain name or public IP
- [ ] Auto-detect public IP as suggestion
- [ ] Validate domain/IP format

### Step 4: Confirmation
- [ ] Display summary of configuration
- [ ] Allow going back to change settings
- [ ] Save settings to database
- [ ] Proceed to main dashboard

---

## Feature Priority

### Phase 1 (MVP)
1. **First-run setup wizard** (sing-box detection, SSH port, server address)
2. Dashboard with status display
3. sing-box detection (NOT installation)
4. User creation with key generation
5. Basic config generation (JSON) with default routing
6. User deletion

### Phase 2
1. QR code generation
2. URI generation
3. Service control (start/stop)
4. Settings management (change port, domain)
5. User enable/disable
6. **Routing presets** (Default, Iran Direct, Iran Block, China Direct)

### Phase 3
1. Log viewing
2. Backup/restore
3. Config history
4. **Custom routing rules**
5. **Rule set management**
6. Advanced settings
7. CLI mode

---

## Related Documentation

- [Architecture](./architecture.md) - System design
- [Database](./database.md) - Data storage details
- [UI Screens](./ui-screens.md) - Screen mockups
- [Configuration](./configuration.md) - Config generation
- [Routing](./routing.md) - Traffic routing configuration
