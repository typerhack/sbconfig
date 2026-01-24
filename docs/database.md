# Database

> SQLite database schema and data models for sbconfig.

## Overview

sbconfig uses SQLite for persistent storage. The database file is located at `/var/lib/sbconfig/sbconfig.db`.

### Why SQLite?
- Single file, easy to backup and restore
- No external database server required
- Excellent performance for this use case
- ACID compliant transactions
- Built-in with Rust via `rusqlite` (bundled feature)

## Schema

### Entity Relationship Diagram

```
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│     users       │       │    settings     │       │   migrations    │
├─────────────────┤       ├─────────────────┤       ├─────────────────┤
│ id (PK)         │       │ key (PK)        │       │ version (PK)    │
│ username        │       │ value           │       │ applied_at      │
│ created_at      │       │ updated_at      │       └─────────────────┘
│ key_type        │       └─────────────────┘
│ public_key      │
│ private_key     │
│ is_active       │
│ notes           │
└────────┬────────┘
         │
         │ 1:1                1:N                    1:N
    ┌────┴─────┬────────────────┼────────────────────┤
    │          │                │                    │
    ▼          ▼                ▼                    ▼
┌─────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│  user_limits    │  │     configs      │  │ user_connections │
├─────────────────┤  ├──────────────────┤  ├──────────────────┤
│ user_id (PK/FK) │  │ id (PK)          │  │ id (PK)          │
│ max_connections │  │ user_id (FK)     │  │ user_id (FK)     │
│ traffic_quota   │  │ platform         │  │ session_id (FK)  │
│ quota_period    │  │ config_json      │  │ connected_at     │
│ quota_reset_at  │  │ uri              │  │ disconnected_at  │
│ created_at      │  │ created_at       │  │ client_ip        │
│ updated_at      │  └──────────────────┘  │ bytes_sent       │
└─────────────────┘                        │ bytes_received   │
                                           │ status           │
┌─────────────────┐                        └────────┬─────────┘
│    sessions     │                                 │
├─────────────────┤                                 │
│ id (PK)         │◄────────────────────────────────┘
│ type            │
│ started_at      │        ┌──────────────────┐
│ ended_at        │        │  traffic_usage   │
│ user_id (FK)    │        ├──────────────────┤
│ metadata        │        │ id (PK)          │
│ status          │        │ user_id (FK)     │
└────────┬────────┘        │ period_start     │
         │                 │ period_end       │
         │ 1:N             │ period_type      │
         ▼                 │ bytes_sent       │
┌─────────────────┐        │ bytes_received   │
│      logs       │        │ connection_count │
├─────────────────┤        │ total_duration   │
│ id (PK)         │        └──────────────────┘
│ timestamp       │
│ session_id (FK) │
│ level           │
│ category        │
│ message         │
│ context         │
│ user_id (FK)    │
└─────────────────┘
```

## Table Definitions

### users

Stores SSH proxy users and their keys.

```sql
CREATE TABLE users (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    username        TEXT UNIQUE NOT NULL,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    key_type        TEXT DEFAULT 'ed25519' CHECK (key_type IN ('ed25519', 'rsa')),
    public_key      TEXT NOT NULL,
    private_key     TEXT NOT NULL,  -- Encrypted with AES-256-GCM
    is_active       BOOLEAN DEFAULT 1,
    notes           TEXT DEFAULT ''
);

-- Indexes
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_is_active ON users(is_active);
```

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Auto-increment primary key |
| `username` | TEXT | Unique system username |
| `created_at` | DATETIME | User creation timestamp |
| `key_type` | TEXT | Key algorithm: `ed25519` or `rsa` |
| `public_key` | TEXT | SSH public key (OpenSSH format) |
| `private_key` | TEXT | Encrypted private key |
| `is_active` | BOOLEAN | Whether user can connect |
| `notes` | TEXT | Optional user notes |

> **Note:** `ssh_port` is removed from users table - all users share the same custom SSH port configured in settings.

### settings

Key-value store for application settings.

```sql
CREATE TABLE settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Trigger to update timestamp
CREATE TRIGGER update_settings_timestamp 
AFTER UPDATE ON settings
BEGIN
    UPDATE settings SET updated_at = CURRENT_TIMESTAMP WHERE key = NEW.key;
END;
```

| Column | Type | Description |
|--------|------|-------------|
| `key` | TEXT | Setting identifier (primary key) |
| `value` | TEXT | Setting value (JSON for complex values) |
| `updated_at` | DATETIME | Last modification time |

**Default Settings:**

| Key | Default Value | Description |
|-----|---------------|-------------|
| `setup_complete` | `false` | Whether first-run setup wizard is complete |
| `operating_mode` | `""` | `development` or `production` |
| `server_domain` | `""` | Server domain name (production mode) |
| `server_ip` | `""` | Server IP address (production mode) |
| `ssh_port` | `null` | Custom SSH port for sing-box (NOT 22, set during setup) |
| `proxy_port` | `10808` | Default client proxy port |
| `dns_servers` | `["8.8.8.8","1.1.1.1"]` | DNS servers for configs |
| `encryption_salt` | `<random>` | Salt for key encryption |
| `routing_preset` | `default` | Active routing preset: `default`, `iran_direct`, `iran_block`, `china_direct`, `custom` |
| `routing_custom_rules` | `[]` | JSON array of custom routing rules |
| `routing_rule_sets` | `[]` | JSON array of enabled rule set tags |

> **Important:** `ssh_port` has no default - it MUST be configured during the first-run setup wizard. User operations are blocked until setup is complete.

### configs

Stores generated client configurations.

```sql
CREATE TABLE configs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    platform    TEXT NOT NULL CHECK (platform IN ('ios', 'android', 'windows', 'macos', 'linux', 'generic')),
    config_json TEXT NOT NULL,
    uri         TEXT NOT NULL,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX idx_configs_user_id ON configs(user_id);
CREATE INDEX idx_configs_platform ON configs(platform);
CREATE INDEX idx_configs_created_at ON configs(created_at);
```

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Auto-increment primary key |
| `user_id` | INTEGER | Foreign key to users table |
| `platform` | TEXT | Target platform |
| `config_json` | TEXT | Full sing-box JSON config |
| `uri` | TEXT | sing-box:// URI |
| `created_at` | DATETIME | Generation timestamp |

### migrations

Tracks applied database migrations.

```sql
CREATE TABLE migrations (
    version     INTEGER PRIMARY KEY,
    applied_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

| Column | Type | Description |
|--------|------|-------------|
| `version` | INTEGER | Migration version number |
| `applied_at` | DATETIME | When migration was applied |

### sessions

Tracks application and user connection sessions for logging.

```sql
CREATE TABLE sessions (
    id              TEXT PRIMARY KEY,      -- Session ID (timestamp_random)
    type            TEXT NOT NULL CHECK (type IN ('app', 'user', 'task', 'api')),
    started_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    ended_at        DATETIME,
    user_id         INTEGER REFERENCES users(id) ON DELETE SET NULL,
    metadata        TEXT,                  -- JSON metadata
    status          TEXT DEFAULT 'active' CHECK (status IN ('active', 'closed', 'error'))
);

-- Indexes
CREATE INDEX idx_sessions_type ON sessions(type);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_started_at ON sessions(started_at);
CREATE INDEX idx_sessions_status ON sessions(status);
```

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT | Unique session ID (format: `YYYYMMDD_HHMMSS_random`) |
| `type` | TEXT | Session type: `app`, `user`, `task`, `api` |
| `started_at` | DATETIME | Session start timestamp |
| `ended_at` | DATETIME | Session end timestamp (NULL if active) |
| `user_id` | INTEGER | Foreign key to users (for user sessions) |
| `metadata` | TEXT | JSON metadata (IP, user agent, etc.) |
| `status` | TEXT | Session status: `active`, `closed`, `error` |

### logs

Stores application log entries for querying and analysis.

```sql
CREATE TABLE logs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp       DATETIME DEFAULT CURRENT_TIMESTAMP,
    session_id      TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    level           INTEGER NOT NULL CHECK (level BETWEEN 0 AND 5),
    category        TEXT NOT NULL,
    message         TEXT NOT NULL,
    context         TEXT,                  -- JSON context/metadata
    user_id         INTEGER REFERENCES users(id) ON DELETE SET NULL
);

-- Indexes for efficient querying
CREATE INDEX idx_logs_timestamp ON logs(timestamp);
CREATE INDEX idx_logs_session_id ON logs(session_id);
CREATE INDEX idx_logs_level ON logs(level);
CREATE INDEX idx_logs_category ON logs(category);
CREATE INDEX idx_logs_user_id ON logs(user_id);

-- Composite index for common queries
CREATE INDEX idx_logs_level_timestamp ON logs(level, timestamp);
```

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Auto-increment primary key |
| `timestamp` | DATETIME | Log entry timestamp |
| `session_id` | TEXT | Foreign key to sessions table |
| `level` | INTEGER | Log level: 0=TRACE, 1=DEBUG, 2=INFO, 3=WARN, 4=ERROR, 5=FATAL |
| `category` | TEXT | Log category (app, db, ssh, user, etc.) |
| `message` | TEXT | Log message |
| `context` | TEXT | JSON context/metadata |
| `user_id` | INTEGER | Associated user ID (optional) |

### user_limits

Stores per-user connection limits and traffic quotas.

```sql
CREATE TABLE user_limits (
    user_id             INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    max_connections     INTEGER DEFAULT 1,      -- 0 = unlimited
    traffic_quota_bytes INTEGER DEFAULT 0,      -- 0 = unlimited (in bytes)
    quota_period        TEXT DEFAULT 'monthly' CHECK (quota_period IN ('daily', 'weekly', 'monthly', 'total')),
    quota_reset_at      DATETIME,               -- Next quota reset time
    created_at          DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at          DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

| Column | Type | Description |
|--------|------|-------------|
| `user_id` | INTEGER | Foreign key to users (1:1 relationship) |
| `max_connections` | INTEGER | Maximum concurrent connections (0 = unlimited) |
| `traffic_quota_bytes` | INTEGER | Traffic quota in bytes (0 = unlimited) |
| `quota_period` | TEXT | Quota reset period: `daily`, `weekly`, `monthly`, `total` |
| `quota_reset_at` | DATETIME | Next quota reset timestamp |
| `created_at` | DATETIME | Record creation time |
| `updated_at` | DATETIME | Last update time |

### user_connections

Tracks active user connections in real-time.

```sql
CREATE TABLE user_connections (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id      TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    connected_at    DATETIME DEFAULT CURRENT_TIMESTAMP,
    disconnected_at DATETIME,
    client_ip       TEXT,
    client_port     INTEGER,
    bytes_sent      INTEGER DEFAULT 0,
    bytes_received  INTEGER DEFAULT 0,
    status          TEXT DEFAULT 'active' CHECK (status IN ('active', 'closed', 'timeout', 'kicked'))
);

-- Indexes
CREATE INDEX idx_user_connections_user_id ON user_connections(user_id);
CREATE INDEX idx_user_connections_session_id ON user_connections(session_id);
CREATE INDEX idx_user_connections_status ON user_connections(status);
CREATE INDEX idx_user_connections_connected_at ON user_connections(connected_at);
```

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Auto-increment primary key |
| `user_id` | INTEGER | Foreign key to users |
| `session_id` | TEXT | Foreign key to sessions |
| `connected_at` | DATETIME | Connection start time |
| `disconnected_at` | DATETIME | Connection end time |
| `client_ip` | TEXT | Client IP address |
| `client_port` | INTEGER | Client port |
| `bytes_sent` | INTEGER | Bytes uploaded by user |
| `bytes_received` | INTEGER | Bytes downloaded by user |
| `status` | TEXT | Connection status |

### traffic_usage

Aggregated traffic usage statistics per user per period.

```sql
CREATE TABLE traffic_usage (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    period_start    DATETIME NOT NULL,
    period_end      DATETIME NOT NULL,
    period_type     TEXT NOT NULL CHECK (period_type IN ('daily', 'weekly', 'monthly')),
    bytes_sent      INTEGER DEFAULT 0,
    bytes_received  INTEGER DEFAULT 0,
    connection_count INTEGER DEFAULT 0,
    total_duration_secs INTEGER DEFAULT 0,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX idx_traffic_usage_user_id ON traffic_usage(user_id);
CREATE INDEX idx_traffic_usage_period ON traffic_usage(period_start, period_end);
CREATE INDEX idx_traffic_usage_type ON traffic_usage(period_type);

-- Unique constraint to prevent duplicate periods
CREATE UNIQUE INDEX idx_traffic_usage_unique ON traffic_usage(user_id, period_start, period_type);
```

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Auto-increment primary key |
| `user_id` | INTEGER | Foreign key to users |
| `period_start` | DATETIME | Period start timestamp |
| `period_end` | DATETIME | Period end timestamp |
| `period_type` | TEXT | Period type: `daily`, `weekly`, `monthly` |
| `bytes_sent` | INTEGER | Total bytes uploaded in period |
| `bytes_received` | INTEGER | Total bytes downloaded in period |
| `connection_count` | INTEGER | Number of connections in period |
| `total_duration_secs` | INTEGER | Total connection time in seconds |
| `created_at` | DATETIME | Record creation time |

## Rust Data Models

### User Model

```rust
#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub key_type: KeyType,
    pub public_key: String,
    pub private_key: String,  // Decrypted
    pub is_active: bool,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub username: String,
    pub key_type: KeyType,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum KeyType {
    Ed25519,
    Rsa,
}
```

> **Note:** Users no longer have individual `ssh_port` - all users share the same custom SSH port from settings.

### Settings Model

```rust
#[derive(Debug, Clone)]
pub struct Settings {
    pub setup_complete: bool,
    pub operating_mode: Option<OperatingMode>,
    pub server_domain: Option<String>,
    pub server_ip: Option<String>,
    pub ssh_port: Option<u16>,  // None until configured
    pub proxy_port: u16,
    pub dns_servers: Vec<String>,
    pub routing_preset: RoutingPreset,
    pub routing_custom_rules: Vec<CustomRule>,
    pub routing_rule_sets: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperatingMode {
    Development,  // Uses localhost
    Production,   // Uses domain/IP
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RoutingPreset {
    Default,
    IranDirect,
    IranBlock,
    ChinaDirect,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    pub rule_type: RuleType,
    pub value: Vec<String>,
    pub outbound: RuleOutbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RuleType {
    Domain,
    DomainSuffix,
    DomainKeyword,
    DomainRegex,
    IpCidr,
    Port,
    Protocol,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RuleOutbound {
    Proxy,   // ssh-out
    Direct,  // direct
    Block,   // block
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            setup_complete: false,
            operating_mode: None,
            server_domain: None,
            server_ip: None,
            ssh_port: None,  // MUST be set during setup
            proxy_port: 10808,
            dns_servers: vec!["8.8.8.8".into(), "1.1.1.1".into()],
            routing_preset: RoutingPreset::Default,
            routing_custom_rules: vec![],
            routing_rule_sets: vec![],
        }
    }
}

impl Settings {
    /// Returns the server address based on operating mode
    pub fn server_address(&self) -> Option<String> {
        match self.operating_mode {
            Some(OperatingMode::Development) => Some("localhost".to_string()),
            Some(OperatingMode::Production) => {
                self.server_domain.clone().or(self.server_ip.clone())
            }
            None => None,
        }
    }
}
```

### Config Model

```rust
#[derive(Debug, Clone)]
pub struct GeneratedConfig {
    pub id: Option<i64>,
    pub user_id: i64,
    pub platform: Platform,
    pub config_json: String,
    pub uri: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    Ios,
    Android,
    Windows,
    MacOs,
    Linux,
    Generic,
}
```

### Session Model

```rust
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub session_type: SessionType,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub user_id: Option<i64>,
    pub metadata: Option<serde_json::Value>,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SessionType {
    App,    // Application session
    User,   // User connection session
    Task,   // Background task session
    Api,    // CLI/API request session
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SessionStatus {
    Active,
    Closed,
    Error,
}

impl Session {
    pub fn new(session_type: SessionType, user_id: Option<i64>) -> Self {
        Self {
            id: Self::generate_id(),
            session_type,
            started_at: Utc::now(),
            ended_at: None,
            user_id,
            metadata: None,
            status: SessionStatus::Active,
        }
    }
    
    fn generate_id() -> String {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let random: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();
        format!("{}_{}", timestamp, random.to_lowercase())
    }
}
```

### Log Model

```rust
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub level: LogLevel,
    pub category: String,
    pub message: String,
    pub context: Option<serde_json::Value>,
    pub user_id: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }
}
```

### User Limits Model

```rust
#[derive(Debug, Clone)]
pub struct UserLimits {
    pub user_id: i64,
    pub max_connections: u32,        // 0 = unlimited
    pub traffic_quota_bytes: u64,    // 0 = unlimited
    pub quota_period: QuotaPeriod,
    pub quota_reset_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuotaPeriod {
    Daily,
    Weekly,
    Monthly,
    Total,  // Never resets
}

impl Default for UserLimits {
    fn default() -> Self {
        Self {
            user_id: 0,
            max_connections: 1,             // Default: 1 connection
            traffic_quota_bytes: 0,         // Default: unlimited
            quota_period: QuotaPeriod::Monthly,
            quota_reset_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl UserLimits {
    /// Check if connection limit allows a new connection
    pub fn can_connect(&self, current_connections: u32) -> bool {
        self.max_connections == 0 || current_connections < self.max_connections
    }
    
    /// Check if traffic quota allows more data
    pub fn has_quota_remaining(&self, used_bytes: u64) -> bool {
        self.traffic_quota_bytes == 0 || used_bytes < self.traffic_quota_bytes
    }
    
    /// Get quota usage percentage (0-100)
    pub fn quota_usage_percent(&self, used_bytes: u64) -> f64 {
        if self.traffic_quota_bytes == 0 {
            0.0
        } else {
            (used_bytes as f64 / self.traffic_quota_bytes as f64) * 100.0
        }
    }
}
```

### Connection Model

```rust
#[derive(Debug, Clone)]
pub struct UserConnection {
    pub id: Option<i64>,
    pub user_id: i64,
    pub session_id: String,
    pub connected_at: DateTime<Utc>,
    pub disconnected_at: Option<DateTime<Utc>>,
    pub client_ip: Option<String>,
    pub client_port: Option<u16>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub status: ConnectionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionStatus {
    Active,
    Closed,
    Timeout,
    Kicked,  // Forcefully disconnected
}

impl UserConnection {
    /// Get connection duration in seconds
    pub fn duration_secs(&self) -> i64 {
        let end = self.disconnected_at.unwrap_or_else(Utc::now);
        (end - self.connected_at).num_seconds()
    }
    
    /// Get total traffic (sent + received)
    pub fn total_bytes(&self) -> u64 {
        self.bytes_sent + self.bytes_received
    }
}
```

### Traffic Usage Model

```rust
#[derive(Debug, Clone)]
pub struct TrafficUsage {
    pub id: Option<i64>,
    pub user_id: i64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub period_type: QuotaPeriod,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_count: u32,
    pub total_duration_secs: i64,
    pub created_at: DateTime<Utc>,
}

impl TrafficUsage {
    /// Get total traffic (sent + received)
    pub fn total_bytes(&self) -> u64 {
        self.bytes_sent + self.bytes_received
    }
    
    /// Format bytes as human-readable string
    pub fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;
        
        if bytes >= TB {
            format!("{:.2} TB", bytes as f64 / TB as f64)
        } else if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}
```

## Database Operations

### Initialization

```rust
impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }
    
    pub fn migrate(&self) -> Result<()> {
        let version = self.get_migration_version()?;
        
        if version < 1 {
            self.conn.execute_batch(include_str!("migrations/001_initial.sql"))?;
            self.set_migration_version(1)?;
        }
        
        // Future migrations...
        
        Ok(())
    }
}
```

### User CRUD

```rust
impl Database {
    pub fn create_user(&self, new_user: &NewUser, keys: &KeyPair) -> Result<User> {
        let encrypted_private = self.encrypt_key(&keys.private_key)?;
        
        self.conn.execute(
            "INSERT INTO users (username, key_type, public_key, private_key, notes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                new_user.username,
                new_user.key_type.to_string(),
                keys.public_key,
                encrypted_private,
                new_user.notes.as_deref().unwrap_or(""),
            ],
        )?;
        
        let id = self.conn.last_insert_rowid();
        self.get_user(id)
    }
    
    pub fn get_users(&self) -> Result<Vec<User>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, username, created_at, key_type, public_key, private_key, is_active, notes
             FROM users ORDER BY created_at DESC"
        )?;
        
        let users = stmt.query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                created_at: row.get(2)?,
                key_type: row.get::<_, String>(3)?.parse().unwrap(),
                public_key: row.get(4)?,
                private_key: self.decrypt_key(&row.get::<_, String>(5)?).unwrap(),
                is_active: row.get(6)?,
                notes: row.get(7)?,
            })
        })?;
        
        users.collect()
    }
    
    pub fn delete_user(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM users WHERE id = ?1", [id])?;
        Ok(())
    }
    
    pub fn toggle_user_active(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE users SET is_active = NOT is_active WHERE id = ?1",
            [id],
        )?;
        Ok(())
    }
}
```

### Settings Operations

```rust
impl Database {
    pub fn get_settings(&self) -> Result<Settings> {
        let mut settings = Settings::default();
        
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        
        for row in rows {
            let (key, value) = row?;
            match key.as_str() {
                "server_domain" => settings.server_domain = value,
                "server_ip" => settings.server_ip = value,
                "ssh_port" => settings.ssh_port = value.parse().unwrap_or(22),
                "proxy_port" => settings.proxy_port = value.parse().unwrap_or(10808),
                "dns_servers" => settings.dns_servers = serde_json::from_str(&value)?,
                _ => {}
            }
        }
        
        Ok(settings)
    }
    
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            [key, value],
        )?;
        Ok(())
    }
}
```

## Encryption

Private keys are encrypted before storage using AES-256-GCM:

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

impl Database {
    fn get_encryption_key(&self) -> Result<[u8; 32]> {
        // Derive from machine-id + stored salt
        let machine_id = std::fs::read_to_string("/etc/machine-id")?;
        let salt = self.get_setting("encryption_salt")?
            .unwrap_or_else(|| {
                let salt = generate_random_salt();
                self.set_setting("encryption_salt", &salt).unwrap();
                salt
            });
        
        // PBKDF2 or Argon2 derivation
        derive_key(&machine_id, &salt)
    }
    
    fn encrypt_key(&self, plaintext: &str) -> Result<String> {
        let key = self.get_encryption_key()?;
        let cipher = Aes256Gcm::new(Key::from_slice(&key));
        let nonce = generate_random_nonce();
        
        let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())?;
        
        // Return as base64: nonce || ciphertext
        Ok(base64::encode([&nonce[..], &ciphertext[..]].concat()))
    }
    
    fn decrypt_key(&self, encrypted: &str) -> Result<String> {
        let key = self.get_encryption_key()?;
        let cipher = Aes256Gcm::new(Key::from_slice(&key));
        
        let data = base64::decode(encrypted)?;
        let (nonce, ciphertext) = data.split_at(12);
        
        let plaintext = cipher.decrypt(Nonce::from_slice(nonce), ciphertext)?;
        
        Ok(String::from_utf8(plaintext)?)
    }
}
```

## Backup Format

Database backups are exported as JSON:

```json
{
  "version": 1,
  "exported_at": "2024-01-15T10:30:00Z",
  "settings": {
    "server_domain": "example.com",
    "ssh_port": 7344
  },
  "users": [
    {
      "username": "user_alpha",
      "created_at": "2024-01-10T08:00:00Z",
      "ssh_port": 7344,
      "key_type": "ed25519",
      "public_key": "ssh-ed25519 AAAA...",
      "private_key": "-----BEGIN OPENSSH PRIVATE KEY-----\n...",
      "is_active": true,
      "notes": ""
    }
  ]
}
```

**Note:** Private keys in backups are stored in plaintext. Backup files should be protected appropriately.

## Related Documentation

- [Architecture](./architecture.md) - System design
- [Features](./features.md) - Feature specifications
- [Development](./development.md) - Dev setup guide
- [Routing](./routing.md) - Routing configuration details
