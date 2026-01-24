# Features

> Complete feature list and specifications for sbconfig.

## Feature Overview

| Category | Features |
|----------|----------|
| sing-box Management | Detection (not install), start/stop, status, version check |
| User Management | Create, list, delete, enable/disable, regenerate keys |
| **Connection Limits** | Max concurrent connections per user, real-time tracking |
| **Traffic Quota** | Data transfer limits, usage tracking, auto-disable on exceed |
| Config Generation | JSON export, URI generation, QR codes, multi-platform |
| **Routing** | Presets (Iran, China), custom rules, rule sets, block/direct/proxy |
| Server Settings | Custom SSH port, domain/localhost, proxy port configuration |
| **Logging** | Session-based logs, levels (TRACE-FATAL), filtering, auto-cleanup |
| Monitoring | Log viewing, service status, connection stats |
| Data Management | SQLite storage, backup, restore |
| **Update** | Check for updates, download and install, rollback on failure |
| **Uninstall** | Complete removal of binary, data, users, SSH config |

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

### Connection Limits
- [ ] Set maximum concurrent connections per user
- [ ] Options: 1, 2, 3, 5, 10, unlimited
- [ ] Real-time connection count tracking
- [ ] Reject new connections when limit reached
- [ ] Display current connections vs limit
- [ ] Alert when approaching limit (80%)
- [ ] Per-user limit override (global default + per-user)
- [ ] Connection history tracking

### Traffic Quota Management
- [ ] Set data transfer limit per user (MB/GB)
- [ ] Options: 1GB, 5GB, 10GB, 50GB, 100GB, unlimited
- [ ] Track upload and download separately
- [ ] Track total traffic (upload + download)
- [ ] Quota period: daily, weekly, monthly, total
- [ ] Auto-disable user when quota exceeded
- [ ] Warning at 80% and 95% quota usage
- [ ] Reset quota manually or automatically
- [ ] Traffic usage history and graphs
- [ ] Per-user quota override

### Usage Statistics
- [ ] Current session duration
- [ ] Total connection time (all time)
- [ ] Connection count (today/week/month/total)
- [ ] Traffic used (today/week/month/total)
- [ ] Last connection timestamp
- [ ] Peak usage times
- [ ] Export usage report (CSV/JSON)

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

## 7. Logging System

> See [docs/logging.md](./logging.md) for detailed logging documentation.

### Log Levels
- [ ] TRACE - Most detailed, step-by-step execution
- [ ] DEBUG - Detailed information for debugging
- [ ] INFO - General operational information
- [ ] WARN - Warning conditions, potential issues
- [ ] ERROR - Error conditions, operation failed
- [ ] FATAL - Critical errors, application may crash
- [ ] Configurable minimum log level

### Log Categories
- [ ] `app` - Application lifecycle
- [ ] `db` - Database operations
- [ ] `ssh` - SSH operations
- [ ] `singbox` - sing-box operations
- [ ] `ui` - User interface
- [ ] `auth` - Authentication
- [ ] `user` - User management
- [ ] `config` - Config generation
- [ ] `session` - Session tracking
- [ ] `system` - System operations

### Session-Based Logging
- [ ] Unique session ID for each app run/user connection
- [ ] Session types: app, user, task, api
- [ ] Separate log files per session
- [ ] Session metadata (user, start/end time, status)
- [ ] Easy filtering by session ID
- [ ] Session lifecycle tracking

### Log Storage
- [ ] Dual storage: SQLite database + log files
- [ ] Indexed for fast querying
- [ ] JSON context/metadata support
- [ ] Session-specific log files in `logs/sessions/`
- [ ] Main application log with rotation
- [ ] Error-only log file

### Log Viewing (TUI)
- [ ] Real-time log streaming
- [ ] Filter by level (checkboxes)
- [ ] Filter by category
- [ ] Filter by session (dropdown)
- [ ] Filter by user
- [ ] Time range picker
- [ ] Full-text search
- [ ] Pause/resume streaming

### Log Querying (CLI)
- [ ] `sbconfig logs` - View recent logs
- [ ] `sbconfig logs --session <id>` - View session logs
- [ ] `sbconfig logs --level error` - Filter by level
- [ ] `sbconfig logs --category user,ssh` - Filter by category
- [ ] `sbconfig logs --user <username>` - Filter by user
- [ ] `sbconfig logs --since "1 hour ago"` - Time filter
- [ ] `sbconfig logs --search "keyword"` - Search logs
- [ ] `sbconfig logs --follow` - Real-time follow
- [ ] `sbconfig logs --export <file>` - Export logs

### Log Rotation
- [ ] Size-based rotation (10MB default)
- [ ] Keep N rotated files (5 default)
- [ ] Compress old logs (gzip)
- [ ] Archive by month

### Log Retention & Cleanup
- [ ] Configurable retention per log level
- [ ] TRACE: 1 day, DEBUG: 3 days, INFO: 7 days
- [ ] WARN: 30 days, ERROR: 90 days, FATAL: 365 days
- [ ] Automatic daily cleanup
- [ ] Manual cleanup command
- [ ] Session log retention: 30 days
- [ ] Archive retention: 90 days

### External Log Sources
- [ ] View sing-box service logs
- [ ] View SSH authentication logs (`/var/log/auth.log`)
- [ ] Aggregate all log sources
- [ ] Filter by source

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

## 13. Update Management

> Built-in update functionality to keep sbconfig up to date.

### Update Check
- [ ] Check for new versions on GitHub releases
- [ ] Display current version vs latest version
- [ ] Show changelog/release notes summary
- [ ] Notify user of available updates on startup (optional)

### Update Process
- [ ] Download new binary from GitHub releases
- [ ] Verify checksum/signature of downloaded binary
- [ ] Backup current binary before replacement
- [ ] Backup database before update
- [ ] Replace binary with new version
- [ ] Verify new binary works (`sbconfig --version`)
- [ ] Rollback to backup if update fails
- [ ] Display update success/failure status

### Update Options
- [ ] Check for updates manually from Settings menu
- [ ] Auto-check for updates on startup (configurable)
- [ ] Skip specific versions
- [ ] View update history

### CLI Update
```
sbconfig update          # Check and install updates
sbconfig update --check  # Check only, don't install
sbconfig update --force  # Force reinstall current version
```

---

## 14. Complete Uninstall

> Safe and complete removal of sbconfig and all its components.

### Uninstall Scope

| Component | Description | Removal Action |
|-----------|-------------|----------------|
| Binary | `/usr/local/bin/sbconfig` | Delete file |
| Database | `/var/lib/sbconfig/sbconfig.db` | Delete file |
| Data Directory | `/var/lib/sbconfig/` | Delete directory |
| Generated Configs | `/var/lib/sbconfig/configs/` | Delete directory |
| Backups | `/var/lib/sbconfig/backups/` | Delete directory |
| System Users | Users created by sbconfig | Delete with `userdel -r` |
| SSH Keys | `~/.ssh/authorized_keys` for each user | Removed with user |
| SSH Port Config | Custom port line in `/etc/ssh/sshd_config` | Remove line |
| Firewall Rules | Custom SSH port rules | Show removal commands |

### Uninstall Process

#### Step 1: Pre-Uninstall Summary
- [ ] Display list of all components that will be removed
- [ ] Show number of users that will be deleted
- [ ] Show database size and backup recommendation
- [ ] Display SSH port that will be deconfigured
- [ ] Warn about irreversible data loss

#### Step 2: Backup Prompt
- [ ] Ask if user wants to create final backup before uninstall
- [ ] If yes: Create timestamped backup to specified location
- [ ] Backup includes: database, configs, user list

#### Step 3: Confirmation
- [ ] Require explicit confirmation (type "UNINSTALL" to confirm)
- [ ] Show final warning about data loss
- [ ] Option to cancel at this point

#### Step 4: Remove System Users
- [ ] List all users created by sbconfig
- [ ] Delete each system user with `userdel -r <username>`
- [ ] Remove home directories and SSH keys
- [ ] Log each user removal

#### Step 5: Remove SSH Configuration
- [ ] Remove custom SSH port from `/etc/ssh/sshd_config`
- [ ] Restart SSH service to apply changes
- [ ] Display firewall rule removal commands (manual step)

#### Step 6: Remove Data
- [ ] Delete database file
- [ ] Delete configs directory
- [ ] Delete backups directory
- [ ] Delete data directory

#### Step 7: Remove Binary
- [ ] Delete `/usr/local/bin/sbconfig`
- [ ] Verify binary is removed

#### Step 8: Post-Uninstall Summary
- [ ] Display summary of removed components
- [ ] Show any manual steps required (firewall rules)
- [ ] Display backup location if backup was created
- [ ] Exit application

### Uninstall Options

| Option | Description |
|--------|-------------|
| Full Uninstall | Remove everything including users and data |
| Keep Users | Remove sbconfig but keep system users intact |
| Keep Data | Remove binary but keep database for reinstall |
| Dry Run | Show what would be removed without removing |

### CLI Uninstall
```
sbconfig uninstall              # Interactive uninstall
sbconfig uninstall --full       # Full uninstall (requires --confirm)
sbconfig uninstall --keep-users # Keep system users
sbconfig uninstall --keep-data  # Keep database and configs
sbconfig uninstall --dry-run    # Show what would be removed
sbconfig uninstall --confirm    # Skip confirmation prompts
```

### Uninstall Script (Standalone)
For cases where sbconfig binary is corrupted or missing:

```bash
#!/bin/bash
# uninstall-sbconfig.sh

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${RED}sbconfig Complete Uninstaller${NC}"
echo "=============================="
echo ""
echo -e "${YELLOW}WARNING: This will remove sbconfig and ALL associated data!${NC}"
echo ""

# Check root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: Please run as root (sudo)${NC}"
    exit 1
fi

# Show what will be removed
echo "The following will be removed:"
echo "  - Binary: /usr/local/bin/sbconfig"
echo "  - Data:   /var/lib/sbconfig/"

# List users from database if exists
if [ -f /var/lib/sbconfig/sbconfig.db ]; then
    echo "  - Database: /var/lib/sbconfig/sbconfig.db"
    # Extract usernames (requires sqlite3)
    if command -v sqlite3 &> /dev/null; then
        USERS=$(sqlite3 /var/lib/sbconfig/sbconfig.db "SELECT username FROM users;" 2>/dev/null || echo "")
        if [ -n "$USERS" ]; then
            echo "  - System users:"
            echo "$USERS" | while read user; do
                echo "      - $user"
            done
        fi
    fi
fi

echo ""
read -p "Type 'UNINSTALL' to confirm: " confirm

if [ "$confirm" != "UNINSTALL" ]; then
    echo "Uninstall cancelled"
    exit 0
fi

echo ""
echo "Starting uninstall..."

# Remove system users created by sbconfig
if [ -f /var/lib/sbconfig/sbconfig.db ] && command -v sqlite3 &> /dev/null; then
    USERS=$(sqlite3 /var/lib/sbconfig/sbconfig.db "SELECT username FROM users;" 2>/dev/null || echo "")
    if [ -n "$USERS" ]; then
        echo "$USERS" | while read user; do
            if id "$user" &>/dev/null; then
                echo "Removing user: $user"
                userdel -r "$user" 2>/dev/null || true
            fi
        done
    fi
fi

# Get SSH port before removing database
SSH_PORT=""
if [ -f /var/lib/sbconfig/sbconfig.db ] && command -v sqlite3 &> /dev/null; then
    SSH_PORT=$(sqlite3 /var/lib/sbconfig/sbconfig.db "SELECT value FROM settings WHERE key='ssh_port';" 2>/dev/null || echo "")
fi

# Remove SSH port from sshd_config
if [ -n "$SSH_PORT" ]; then
    if grep -q "^Port $SSH_PORT" /etc/ssh/sshd_config; then
        echo "Removing SSH port $SSH_PORT from sshd_config"
        sed -i "/^Port $SSH_PORT$/d" /etc/ssh/sshd_config
        systemctl restart sshd 2>/dev/null || service ssh restart 2>/dev/null || true
    fi
fi

# Remove data directory
if [ -d /var/lib/sbconfig ]; then
    echo "Removing data directory"
    rm -rf /var/lib/sbconfig
fi

# Remove binary
if [ -f /usr/local/bin/sbconfig ]; then
    echo "Removing binary"
    rm -f /usr/local/bin/sbconfig
fi

# Remove backup binary if exists
if [ -f /usr/local/bin/sbconfig.bak ]; then
    rm -f /usr/local/bin/sbconfig.bak
fi

echo ""
echo -e "${GREEN}Uninstall complete!${NC}"
echo ""
echo "Manual steps (if applicable):"
if [ -n "$SSH_PORT" ]; then
    echo "  - Remove firewall rule for port $SSH_PORT"
    echo "    UFW:       sudo ufw delete allow $SSH_PORT/tcp"
    echo "    firewalld: sudo firewall-cmd --permanent --remove-port=$SSH_PORT/tcp && sudo firewall-cmd --reload"
fi
```

---

## Feature Priority

### Phase 1 (MVP)
1. **First-run setup wizard** (sing-box detection, SSH port, server address)
2. Dashboard with status display
3. sing-box detection (NOT installation)
4. User creation with key generation
5. Basic config generation (JSON) with default routing
6. User deletion
7. **Basic logging** (INFO level, file-based)

### Phase 2
1. QR code generation
2. URI generation
3. Service control (start/stop)
4. Settings management (change port, domain)
5. User enable/disable
6. **Routing presets** (Default, Iran Direct, Iran Block, China Direct)
7. **Update check and install**
8. **Connection limits** (max concurrent connections per user)
9. **Traffic quota** (basic usage tracking)

### Phase 3
1. **Advanced logging** (session-based, all levels, TUI viewer)
2. Backup/restore
3. Config history
4. **Custom routing rules**
5. **Rule set management**
6. Advanced settings
7. CLI mode
8. **Complete uninstall with cleanup**
9. **Traffic analytics and reporting**
10. **Log retention and auto-cleanup**

---

## Related Documentation

- [Architecture](./architecture.md) - System design
- [Database](./database.md) - Data storage details
- [UI Screens](./ui-screens.md) - Screen mockups
- [Configuration](./configuration.md) - Config generation
- [Routing](./routing.md) - Traffic routing configuration
- [Logging](./logging.md) - Logging system documentation
