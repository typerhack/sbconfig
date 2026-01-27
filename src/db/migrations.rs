// src/db/migrations.rs
// Simple versioned migrations runner

use crate::error::Result;
use rusqlite::{params, Connection};

use super::schema::BASE_SCHEMA;

struct Migration {
    version: i32,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sql: BASE_SCHEMA,
    },
    Migration {
        version: 2,
        sql: r#"
            ALTER TABLE users ADD COLUMN email TEXT;
            CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users(email);
        "#,
    },
    Migration {
        version: 3,
        sql: r#"
            -- Insert default client config settings if they don't exist
            INSERT OR IGNORE INTO settings (key, value) VALUES ('client_routing_preset', 'default');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('client_dns_servers', '8.8.8.8,1.1.1.1');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('client_block_ads', 'true');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('client_proxy_port', '10808');

            -- Insert default server config settings if they don't exist
            INSERT OR IGNORE INTO settings (key, value) VALUES ('server_routing_preset', 'default');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('server_dns_servers', '8.8.8.8,1.1.1.1');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('server_block_ads', 'false');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('server_block_iran', 'false');

            -- Migrate old config_* settings to client_* if they exist
            UPDATE settings SET key = 'client_routing_preset'
            WHERE key = 'config_routing_preset' AND NOT EXISTS (
                SELECT 1 FROM settings WHERE key = 'client_routing_preset'
            );

            UPDATE settings SET key = 'client_dns_servers'
            WHERE key = 'config_dns_servers' AND NOT EXISTS (
                SELECT 1 FROM settings WHERE key = 'client_dns_servers'
            );

            UPDATE settings SET key = 'client_block_ads'
            WHERE key = 'config_iran_adblock_enabled' AND NOT EXISTS (
                SELECT 1 FROM settings WHERE key = 'client_block_ads'
            );
        "#,
    },
];

pub fn run_migrations(conn: &mut Connection) -> Result<()> {
    // Ensure migrations table exists before tracking versions.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS migrations (
            id INTEGER PRIMARY KEY,
            version INTEGER NOT NULL UNIQUE,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let applied = {
        let mut stmt = conn.prepare("SELECT version FROM migrations ORDER BY version")?;
        let rows = stmt.query_map([], |row| row.get::<_, i32>(0))?;
        let mut applied = std::collections::HashSet::new();
        for row in rows {
            applied.insert(row?);
        }
        applied
    };

    let tx = conn.transaction()?;
    for migration in MIGRATIONS {
        if applied.contains(&migration.version) {
            continue;
        }
        tx.execute_batch(migration.sql)?;
        tx.execute(
            "INSERT INTO migrations (version) VALUES (?)",
            params![migration.version],
        )?;
    }
    tx.commit()?;

    Ok(())
}
