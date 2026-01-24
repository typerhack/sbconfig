# Logging System

> Comprehensive logging system for sbconfig with session-based logs, log levels, filtering, and automatic cleanup.

## Overview

sbconfig implements a structured logging system that:
- Logs to both files and database for queryability
- Separates logs by session for easy debugging
- Supports multiple log levels
- Provides filtering and search capabilities
- Auto-removes old logs based on retention policy

## Log Levels

| Level | Code | Description | Use Case |
|-------|------|-------------|----------|
| `TRACE` | 0 | Most detailed, step-by-step execution | Deep debugging, tracing code flow |
| `DEBUG` | 1 | Detailed information for debugging | Development, troubleshooting |
| `INFO` | 2 | General operational information | Normal operation events |
| `WARN` | 3 | Warning conditions, not errors | Potential issues, degraded performance |
| `ERROR` | 4 | Error conditions, operation failed | Recoverable errors |
| `FATAL` | 5 | Critical errors, application may crash | Unrecoverable errors |

### Log Level Configuration

```rust
#[derive(Debug, Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}
```

Default log level: `INFO` (logs INFO, WARN, ERROR, FATAL)

## Log Categories

Logs are categorized by component/module:

| Category | Description | Examples |
|----------|-------------|----------|
| `app` | Application lifecycle | Startup, shutdown, config load |
| `db` | Database operations | Queries, migrations, connections |
| `ssh` | SSH operations | Key generation, user creation |
| `singbox` | sing-box operations | Service control, config generation |
| `ui` | User interface | Screen navigation, user input |
| `auth` | Authentication | Login attempts, permission checks |
| `user` | User management | Create, delete, modify users |
| `config` | Config generation | JSON/URI generation, export |
| `session` | Session tracking | Connection events, traffic |
| `system` | System operations | File I/O, process management |

## Session-Based Logging

### Session Concept

Each user connection or application run creates a **session**. Sessions group related log entries for easier debugging.

```
Session ID Format: {timestamp}_{random_id}
Example: 20240122_153045_a1b2c3
```

### Session Types

| Type | Description | Created When |
|------|-------------|--------------|
| `app` | Application session | sbconfig starts |
| `user` | User connection session | User connects via SSH |
| `task` | Background task session | Scheduled job runs |
| `api` | API request session | CLI command executed |

### Session Lifecycle

```
┌─────────────────────────────────────────────────────────────────┐
│                      Session Lifecycle                          │
└─────────────────────────────────────────────────────────────────┘

  Session Start                                      Session End
       │                                                  │
       ▼                                                  ▼
  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────────────┐
  │ Created │───▶│ Active  │───▶│ Closing │───▶│ Closed/Archived │
  └─────────┘    └─────────┘    └─────────┘    └─────────────────┘
       │              │              │                   │
       │              │              │                   │
       ▼              ▼              ▼                   ▼
   Log: Session   Log: Events    Log: Summary      Move to archive
   started        during         stats             or delete
                  session
```

## Log Storage

### Dual Storage Strategy

Logs are stored in two locations:

1. **SQLite Database** - For querying, filtering, and analytics
2. **Log Files** - For traditional file-based access and external tools

### Database Schema

```sql
-- Log entries table
CREATE TABLE logs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp       DATETIME DEFAULT CURRENT_TIMESTAMP,
    session_id      TEXT NOT NULL,
    level           INTEGER NOT NULL,  -- 0=TRACE to 5=FATAL
    category        TEXT NOT NULL,
    message         TEXT NOT NULL,
    context         TEXT,              -- JSON metadata
    user_id         INTEGER,           -- FK to users (optional)
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- Indexes for efficient querying
CREATE INDEX idx_logs_timestamp ON logs(timestamp);
CREATE INDEX idx_logs_session_id ON logs(session_id);
CREATE INDEX idx_logs_level ON logs(level);
CREATE INDEX idx_logs_category ON logs(category);
CREATE INDEX idx_logs_user_id ON logs(user_id);

-- Sessions table
CREATE TABLE sessions (
    id              TEXT PRIMARY KEY,  -- Session ID
    type            TEXT NOT NULL,     -- app, user, task, api
    started_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    ended_at        DATETIME,
    user_id         INTEGER,           -- FK to users (for user sessions)
    metadata        TEXT,              -- JSON metadata
    status          TEXT DEFAULT 'active',  -- active, closed, error
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_sessions_type ON sessions(type);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_started_at ON sessions(started_at);
```

### File Structure

```
/var/lib/sbconfig/logs/
├── current/                    # Active log files
│   ├── sbconfig.log           # Main application log (rotated)
│   ├── error.log              # Error-only log
│   └── sessions/              # Session-specific logs
│       ├── 20240122_153045_a1b2c3.log
│       ├── 20240122_160012_d4e5f6.log
│       └── ...
├── archive/                    # Archived logs (compressed)
│   ├── 2024-01/
│   │   ├── sbconfig_20240115.log.gz
│   │   └── sessions/
│   │       └── ...
│   └── 2024-02/
│       └── ...
└── retention.json             # Retention policy config
```

### Log File Format

```
[TIMESTAMP] [LEVEL] [SESSION_ID] [CATEGORY] MESSAGE {CONTEXT}

Example:
[2024-01-22T15:30:45.123Z] [INFO] [20240122_153045_a1b2c3] [user] User 'user_alpha' created successfully {"user_id": 5, "key_type": "ed25519"}
[2024-01-22T15:30:46.456Z] [ERROR] [20240122_153045_a1b2c3] [ssh] Failed to create SSH key {"error": "permission denied", "path": "/home/user_alpha/.ssh"}
```

## Log Rotation and Retention

### Rotation Policy

| File | Max Size | Max Files | Rotation |
|------|----------|-----------|----------|
| `sbconfig.log` | 10 MB | 5 | Size-based |
| `error.log` | 5 MB | 10 | Size-based |
| Session logs | 1 MB | N/A | Per-session |

### Retention Policy

```json
{
  "retention": {
    "database": {
      "trace": "1 day",
      "debug": "3 days",
      "info": "7 days",
      "warn": "30 days",
      "error": "90 days",
      "fatal": "365 days"
    },
    "files": {
      "current": "7 days",
      "archive": "90 days",
      "sessions": "30 days"
    }
  },
  "cleanup": {
    "schedule": "daily",
    "time": "03:00"
  }
}
```

### Auto-Cleanup Process

```rust
pub async fn cleanup_logs(retention: &RetentionPolicy) -> Result<CleanupStats> {
    let mut stats = CleanupStats::default();
    
    // 1. Clean database logs by level
    for (level, max_age) in &retention.database {
        let cutoff = Utc::now() - *max_age;
        let deleted = db.execute(
            "DELETE FROM logs WHERE level = ? AND timestamp < ?",
            [level, cutoff]
        )?;
        stats.db_deleted += deleted;
    }
    
    // 2. Archive old session logs
    for session_file in read_dir("logs/current/sessions")? {
        if session_file.age() > retention.files.sessions {
            compress_and_move(session_file, "logs/archive")?;
            stats.files_archived += 1;
        }
    }
    
    // 3. Delete old archives
    for archive in read_dir("logs/archive")? {
        if archive.age() > retention.files.archive {
            remove_file(archive)?;
            stats.files_deleted += 1;
        }
    }
    
    Ok(stats)
}
```

## Querying Logs

### CLI Commands

```bash
# View recent logs
sbconfig logs

# View logs for specific session
sbconfig logs --session 20240122_153045_a1b2c3

# Filter by level
sbconfig logs --level error
sbconfig logs --level warn --and-above

# Filter by category
sbconfig logs --category user
sbconfig logs --category ssh,auth

# Filter by user
sbconfig logs --user user_alpha
sbconfig logs --user-id 5

# Filter by time range
sbconfig logs --since "2024-01-22 15:00:00"
sbconfig logs --since "1 hour ago"
sbconfig logs --from "2024-01-20" --to "2024-01-22"

# Search in messages
sbconfig logs --search "connection failed"
sbconfig logs --search "user_alpha" --category auth

# Output formats
sbconfig logs --format json
sbconfig logs --format csv
sbconfig logs --format table

# Export logs
sbconfig logs --export logs_export.json --since "1 week ago"

# Follow logs in real-time
sbconfig logs --follow
sbconfig logs --follow --level error
```

### TUI Log Viewer

The TUI provides an interactive log viewer with:
- Real-time log streaming
- Level filtering (checkboxes)
- Category filtering
- Session selection dropdown
- User filtering
- Time range picker
- Full-text search
- Export functionality

### Database Queries

```sql
-- Get all errors in the last hour
SELECT * FROM logs 
WHERE level >= 4 
AND timestamp > datetime('now', '-1 hour')
ORDER BY timestamp DESC;

-- Get logs for a specific session
SELECT * FROM logs 
WHERE session_id = '20240122_153045_a1b2c3'
ORDER BY timestamp ASC;

-- Get user activity
SELECT l.* FROM logs l
JOIN sessions s ON l.session_id = s.id
WHERE s.user_id = 5
ORDER BY l.timestamp DESC
LIMIT 100;

-- Count errors by category (last 24h)
SELECT category, COUNT(*) as error_count
FROM logs
WHERE level >= 4
AND timestamp > datetime('now', '-1 day')
GROUP BY category
ORDER BY error_count DESC;

-- Get session summary
SELECT 
    s.id,
    s.type,
    s.started_at,
    s.ended_at,
    COUNT(l.id) as log_count,
    SUM(CASE WHEN l.level >= 4 THEN 1 ELSE 0 END) as error_count
FROM sessions s
LEFT JOIN logs l ON s.id = l.session_id
GROUP BY s.id
ORDER BY s.started_at DESC;
```

## Rust Implementation

### Logger Interface

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub struct Logger {
    session_id: String,
    min_level: LogLevel,
    db: Arc<Database>,
    file_writer: Arc<Mutex<FileWriter>>,
}

impl Logger {
    pub fn new(session_id: &str, min_level: LogLevel) -> Self {
        // ...
    }
    
    pub fn trace(&self, category: &str, message: &str) {
        self.log(LogLevel::Trace, category, message, None);
    }
    
    pub fn debug(&self, category: &str, message: &str) {
        self.log(LogLevel::Debug, category, message, None);
    }
    
    pub fn info(&self, category: &str, message: &str) {
        self.log(LogLevel::Info, category, message, None);
    }
    
    pub fn warn(&self, category: &str, message: &str) {
        self.log(LogLevel::Warn, category, message, None);
    }
    
    pub fn error(&self, category: &str, message: &str) {
        self.log(LogLevel::Error, category, message, None);
    }
    
    pub fn fatal(&self, category: &str, message: &str) {
        self.log(LogLevel::Fatal, category, message, None);
    }
    
    pub fn log_with_context<T: Serialize>(
        &self,
        level: LogLevel,
        category: &str,
        message: &str,
        context: &T,
    ) {
        let ctx = serde_json::to_value(context).ok();
        self.log(level, category, message, ctx);
    }
    
    fn log(
        &self,
        level: LogLevel,
        category: &str,
        message: &str,
        context: Option<serde_json::Value>,
    ) {
        if level < self.min_level {
            return;
        }
        
        let entry = LogEntry {
            id: None,
            timestamp: Utc::now(),
            session_id: self.session_id.clone(),
            level,
            category: category.to_string(),
            message: message.to_string(),
            context,
            user_id: None,
        };
        
        // Write to database (async)
        self.db.insert_log(&entry);
        
        // Write to file
        self.file_writer.lock().unwrap().write(&entry);
    }
}
```

### Usage Examples

```rust
// Create logger for session
let logger = Logger::new("20240122_153045_a1b2c3", LogLevel::Info);

// Simple logging
logger.info("app", "Application started");
logger.warn("db", "Database connection slow");
logger.error("ssh", "Failed to generate SSH key");

// Logging with context
logger.log_with_context(
    LogLevel::Info,
    "user",
    "User created successfully",
    &json!({
        "user_id": 5,
        "username": "user_alpha",
        "key_type": "ed25519"
    }),
);

// Error with context
logger.log_with_context(
    LogLevel::Error,
    "config",
    "Config generation failed",
    &json!({
        "user_id": 5,
        "platform": "ios",
        "error": "Invalid routing rules"
    }),
);
```

### Macros for Convenience

```rust
// Define logging macros
macro_rules! log_trace {
    ($logger:expr, $cat:expr, $($arg:tt)*) => {
        $logger.trace($cat, &format!($($arg)*))
    };
}

macro_rules! log_debug {
    ($logger:expr, $cat:expr, $($arg:tt)*) => {
        $logger.debug($cat, &format!($($arg)*))
    };
}

macro_rules! log_info {
    ($logger:expr, $cat:expr, $($arg:tt)*) => {
        $logger.info($cat, &format!($($arg)*))
    };
}

macro_rules! log_warn {
    ($logger:expr, $cat:expr, $($arg:tt)*) => {
        $logger.warn($cat, &format!($($arg)*))
    };
}

macro_rules! log_error {
    ($logger:expr, $cat:expr, $($arg:tt)*) => {
        $logger.error($cat, &format!($($arg)*))
    };
}

macro_rules! log_fatal {
    ($logger:expr, $cat:expr, $($arg:tt)*) => {
        $logger.fatal($cat, &format!($($arg)*))
    };
}

// Usage
log_info!(logger, "user", "Creating user: {}", username);
log_error!(logger, "ssh", "SSH key generation failed: {}", err);
```

## Session Management

### Creating Sessions

```rust
pub struct SessionManager {
    db: Arc<Database>,
}

impl SessionManager {
    pub fn start_session(&self, session_type: SessionType, user_id: Option<i64>) -> Session {
        let session_id = Self::generate_session_id();
        
        let session = Session {
            id: session_id.clone(),
            session_type,
            started_at: Utc::now(),
            ended_at: None,
            user_id,
            metadata: None,
            status: SessionStatus::Active,
        };
        
        self.db.insert_session(&session);
        
        // Create session log file
        self.create_session_log_file(&session_id);
        
        session
    }
    
    pub fn end_session(&self, session_id: &str, status: SessionStatus) {
        self.db.update_session(session_id, |s| {
            s.ended_at = Some(Utc::now());
            s.status = status;
        });
        
        // Flush and close session log file
        self.close_session_log_file(session_id);
    }
    
    fn generate_session_id() -> String {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let random: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();
        format!("{}_{}", timestamp, random.to_lowercase())
    }
}
```

### Session-Aware Logging

```rust
// Each user connection gets its own session
pub async fn handle_user_connection(user: &User) {
    let session = session_manager.start_session(
        SessionType::User,
        Some(user.id),
    );
    
    let logger = Logger::new(&session.id, LogLevel::Debug);
    
    logger.info("session", &format!("User {} connected", user.username));
    
    // ... handle connection ...
    
    logger.info("session", &format!("User {} disconnected", user.username));
    
    session_manager.end_session(&session.id, SessionStatus::Closed);
}
```

## Configuration

### Settings

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Minimum log level to record
    pub min_level: LogLevel,
    
    /// Enable database logging
    pub db_enabled: bool,
    
    /// Enable file logging
    pub file_enabled: bool,
    
    /// Enable session-specific log files
    pub session_files_enabled: bool,
    
    /// Log file rotation settings
    pub rotation: RotationConfig,
    
    /// Retention policy
    pub retention: RetentionConfig,
    
    /// Categories to exclude from logging
    pub excluded_categories: Vec<String>,
    
    /// Enable real-time log streaming to TUI
    pub realtime_enabled: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            min_level: LogLevel::Info,
            db_enabled: true,
            file_enabled: true,
            session_files_enabled: true,
            rotation: RotationConfig::default(),
            retention: RetentionConfig::default(),
            excluded_categories: vec![],
            realtime_enabled: true,
        }
    }
}
```

### Database Settings

```sql
-- Logging settings stored in settings table
INSERT INTO settings (key, value) VALUES
    ('log_level', 'info'),
    ('log_db_enabled', 'true'),
    ('log_file_enabled', 'true'),
    ('log_session_files', 'true'),
    ('log_retention_days', '30'),
    ('log_cleanup_time', '03:00');
```

## Monitoring and Alerts

### Error Rate Monitoring

```rust
pub struct LogMonitor {
    error_threshold: u32,      // Errors per minute to trigger alert
    window_minutes: u32,       // Monitoring window
}

impl LogMonitor {
    pub async fn check_error_rate(&self) -> Option<Alert> {
        let count = self.db.query_scalar::<i64>(
            "SELECT COUNT(*) FROM logs 
             WHERE level >= 4 
             AND timestamp > datetime('now', '-? minutes')",
            [self.window_minutes],
        )?;
        
        if count > self.error_threshold as i64 {
            Some(Alert {
                level: AlertLevel::Warning,
                message: format!(
                    "High error rate: {} errors in last {} minutes",
                    count, self.window_minutes
                ),
            })
        } else {
            None
        }
    }
}
```

## Related Documentation

- [Features](./features.md) - Feature specifications
- [Database](./database.md) - Database schema including log tables
- [Architecture](./architecture.md) - System design
- [UI Screens](./ui-screens.md) - Log viewer UI mockups
