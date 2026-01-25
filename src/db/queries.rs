// src/db/queries.rs
// Database queries
#![allow(dead_code)]

use super::{
    encryption::encrypt, Config, Database, LogEntry, Session, TrafficUsage, User, UserConnection,
    UserLimits,
};
use crate::error::Result;
use rusqlite::{params, OptionalExtension, Row};

/// Helper function to map a row to a User
fn row_to_user(row: &Row) -> rusqlite::Result<User> {
    Ok(User {
        id: row.get(0)?,
        username: row.get(1)?,
        public_key: row.get(2)?,
        private_key_encrypted: row.get(3)?,
        key_type: row.get(4)?,
        is_active: row.get::<_, i64>(5)? != 0,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn row_to_user_limits(row: &Row) -> rusqlite::Result<UserLimits> {
    Ok(UserLimits {
        user_id: row.get(0)?,
        max_connections: row.get(1)?,
        traffic_quota_bytes: row.get(2)?,
        expires_at: row.get(3)?,
    })
}

fn row_to_traffic_usage(row: &Row) -> rusqlite::Result<TrafficUsage> {
    Ok(TrafficUsage {
        id: row.get(0)?,
        user_id: row.get(1)?,
        bytes_up: row.get(2)?,
        bytes_down: row.get(3)?,
        recorded_at: row.get(4)?,
    })
}

fn row_to_config(row: &Row) -> rusqlite::Result<Config> {
    Ok(Config {
        id: row.get(0)?,
        user_id: row.get(1)?,
        platform: row.get(2)?,
        routing_preset: row.get(3)?,
        config_json: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn row_to_session(row: &Row) -> rusqlite::Result<Session> {
    Ok(Session {
        id: row.get(0)?,
        started_at: row.get(1)?,
        ended_at: row.get(2)?,
    })
}

fn row_to_log_entry(row: &Row) -> rusqlite::Result<LogEntry> {
    Ok(LogEntry {
        id: row.get(0)?,
        session_id: row.get(1)?,
        level: row.get(2)?,
        target: row.get(3)?,
        message: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn row_to_user_connection(row: &Row) -> rusqlite::Result<UserConnection> {
    Ok(UserConnection {
        id: row.get(0)?,
        user_id: row.get(1)?,
        connected_at: row.get(2)?,
        remote_ip: row.get(3)?,
        is_active: row.get::<_, i64>(4)? != 0,
    })
}

impl Database {
    // Settings

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn()
            .prepare("SELECT value FROM settings WHERE key = ?")?;
        let result: Option<String> = stmt.query_row(params![key], |row| row.get(0)).optional()?;
        Ok(result)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn().execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
            params![key, value],
        )?;
        Ok(())
    }

    // Users

    #[allow(dead_code)]
    pub fn create_user(
        &self,
        username: &str,
        public_key: &str,
        private_key_encrypted: &str,
        key_type: &str,
    ) -> Result<i64> {
        self.conn().execute(
            "INSERT INTO users (username, public_key, private_key_encrypted, key_type) VALUES (?, ?, ?, ?)",
            params![username, public_key, private_key_encrypted, key_type],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    #[allow(dead_code)]
    pub fn create_user_encrypted(
        &self,
        username: &str,
        public_key: &str,
        private_key: &str,
        key_type: &str,
        password: &str,
    ) -> Result<i64> {
        let encrypted = encrypt(private_key, password)?;
        self.create_user(username, public_key, &encrypted, key_type)
    }

    #[allow(dead_code)]
    pub fn get_user(&self, id: i64) -> Result<Option<User>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, username, public_key, private_key_encrypted, key_type, is_active, created_at, updated_at
             FROM users WHERE id = ?"
        )?;
        let result = stmt.query_row(params![id], row_to_user).optional()?;
        Ok(result)
    }

    #[allow(dead_code)]
    pub fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, username, public_key, private_key_encrypted, key_type, is_active, created_at, updated_at
             FROM users WHERE username = ?"
        )?;
        let result = stmt.query_row(params![username], row_to_user).optional()?;
        Ok(result)
    }

    pub fn list_users(&self) -> Result<Vec<User>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, username, public_key, private_key_encrypted, key_type, is_active, created_at, updated_at
             FROM users ORDER BY created_at DESC"
        )?;
        let users = stmt
            .query_map([], row_to_user)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(users)
    }

    #[allow(dead_code)]
    pub fn delete_user(&self, id: i64) -> Result<bool> {
        let rows = self
            .conn()
            .execute("DELETE FROM users WHERE id = ?", params![id])?;
        Ok(rows > 0)
    }

    pub fn toggle_user_active(&self, id: i64) -> Result<bool> {
        let rows = self.conn().execute(
            "UPDATE users SET is_active = NOT is_active, updated_at = datetime('now') WHERE id = ?",
            params![id],
        )?;
        Ok(rows > 0)
    }

    pub fn count_users(&self) -> Result<i64> {
        let count: i64 = self
            .conn()
            .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn count_active_users(&self) -> Result<i64> {
        let count: i64 = self.conn().query_row(
            "SELECT COUNT(*) FROM users WHERE is_active = 1",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    // User limits

    pub fn set_user_limits(
        &self,
        user_id: i64,
        max_connections: Option<i64>,
        traffic_quota_bytes: Option<i64>,
        expires_at: Option<&str>,
    ) -> Result<()> {
        self.conn().execute(
            "INSERT INTO user_limits (user_id, max_connections, traffic_quota_bytes, expires_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(user_id) DO UPDATE SET
               max_connections = excluded.max_connections,
               traffic_quota_bytes = excluded.traffic_quota_bytes,
               expires_at = excluded.expires_at",
            params![user_id, max_connections, traffic_quota_bytes, expires_at],
        )?;
        Ok(())
    }

    pub fn get_user_limits(&self, user_id: i64) -> Result<Option<UserLimits>> {
        let mut stmt = self.conn().prepare(
            "SELECT user_id, max_connections, traffic_quota_bytes, expires_at
             FROM user_limits WHERE user_id = ?",
        )?;
        let result = stmt.query_row(params![user_id], row_to_user_limits).optional()?;
        Ok(result)
    }

    pub fn delete_user_limits(&self, user_id: i64) -> Result<bool> {
        let rows = self
            .conn()
            .execute("DELETE FROM user_limits WHERE user_id = ?", params![user_id])?;
        Ok(rows > 0)
    }

    // Traffic usage

    pub fn record_traffic_usage(
        &self,
        user_id: i64,
        bytes_up: i64,
        bytes_down: i64,
    ) -> Result<i64> {
        self.conn().execute(
            "INSERT INTO traffic_usage (user_id, bytes_up, bytes_down) VALUES (?, ?, ?)",
            params![user_id, bytes_up, bytes_down],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    pub fn list_traffic_usage(&self, user_id: i64, limit: i64) -> Result<Vec<TrafficUsage>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, user_id, bytes_up, bytes_down, recorded_at
             FROM traffic_usage WHERE user_id = ?
             ORDER BY recorded_at DESC
             LIMIT ?",
        )?;
        let rows = stmt
            .query_map(params![user_id, limit], row_to_traffic_usage)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn traffic_usage_totals(&self, user_id: i64) -> Result<(i64, i64)> {
        let (up, down) = self.conn().query_row(
            "SELECT COALESCE(SUM(bytes_up), 0), COALESCE(SUM(bytes_down), 0)
             FROM traffic_usage WHERE user_id = ?",
            params![user_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        Ok((up, down))
    }

    pub fn clear_traffic_usage(&self, user_id: i64) -> Result<bool> {
        let rows = self.conn().execute(
            "DELETE FROM traffic_usage WHERE user_id = ?",
            params![user_id],
        )?;
        Ok(rows > 0)
    }

    // User connections

    pub fn add_user_connection(
        &self,
        user_id: i64,
        remote_ip: Option<&str>,
    ) -> Result<i64> {
        self.conn().execute(
            "INSERT INTO user_connections (user_id, remote_ip) VALUES (?, ?)",
            params![user_id, remote_ip],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    pub fn set_connection_inactive(&self, id: i64) -> Result<bool> {
        let rows = self.conn().execute(
            "UPDATE user_connections SET is_active = 0 WHERE id = ?",
            params![id],
        )?;
        Ok(rows > 0)
    }

    pub fn list_active_connections(&self, user_id: i64) -> Result<Vec<UserConnection>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, user_id, connected_at, remote_ip, is_active
             FROM user_connections WHERE user_id = ? AND is_active = 1
             ORDER BY connected_at DESC",
        )?;
        let rows = stmt
            .query_map(params![user_id], row_to_user_connection)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn count_active_connections(&self, user_id: i64) -> Result<i64> {
        let count: i64 = self.conn().query_row(
            "SELECT COUNT(*) FROM user_connections WHERE user_id = ? AND is_active = 1",
            params![user_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn clear_connections(&self, user_id: i64) -> Result<bool> {
        let rows = self.conn().execute(
            "DELETE FROM user_connections WHERE user_id = ?",
            params![user_id],
        )?;
        Ok(rows > 0)
    }

    // Configs

    pub fn create_config(
        &self,
        user_id: i64,
        platform: &str,
        routing_preset: &str,
        config_json: &str,
    ) -> Result<i64> {
        self.conn().execute(
            "INSERT INTO configs (user_id, platform, routing_preset, config_json)
             VALUES (?, ?, ?, ?)",
            params![user_id, platform, routing_preset, config_json],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    pub fn get_config(&self, id: i64) -> Result<Option<Config>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, user_id, platform, routing_preset, config_json, created_at
             FROM configs WHERE id = ?",
        )?;
        let result = stmt.query_row(params![id], row_to_config).optional()?;
        Ok(result)
    }

    pub fn list_configs_for_user(&self, user_id: i64) -> Result<Vec<Config>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, user_id, platform, routing_preset, config_json, created_at
             FROM configs WHERE user_id = ?
             ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map(params![user_id], row_to_config)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn delete_config(&self, id: i64) -> Result<bool> {
        let rows = self
            .conn()
            .execute("DELETE FROM configs WHERE id = ?", params![id])?;
        Ok(rows > 0)
    }

    // Sessions

    pub fn start_session(&self, id: &str) -> Result<()> {
        self.conn().execute(
            "INSERT INTO sessions (id) VALUES (?)",
            params![id],
        )?;
        Ok(())
    }

    pub fn end_session(&self, id: &str) -> Result<bool> {
        let rows = self.conn().execute(
            "UPDATE sessions SET ended_at = datetime('now') WHERE id = ?",
            params![id],
        )?;
        Ok(rows > 0)
    }

    pub fn get_session(&self, id: &str) -> Result<Option<Session>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, started_at, ended_at FROM sessions WHERE id = ?",
        )?;
        let result = stmt.query_row(params![id], row_to_session).optional()?;
        Ok(result)
    }

    #[allow(dead_code)]
    pub fn list_sessions(&self, limit: i64) -> Result<Vec<Session>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, started_at, ended_at
             FROM sessions ORDER BY started_at DESC LIMIT ?",
        )?;
        let rows = stmt
            .query_map(params![limit], row_to_session)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // Logs

    pub fn add_log(
        &self,
        session_id: Option<&str>,
        level: &str,
        target: Option<&str>,
        message: &str,
    ) -> Result<i64> {
        self.conn().execute(
            "INSERT INTO logs (session_id, level, target, message)
             VALUES (?, ?, ?, ?)",
            params![session_id, level, target, message],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    pub fn list_logs_for_session(&self, session_id: &str, limit: i64) -> Result<Vec<LogEntry>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, session_id, level, target, message, created_at
             FROM logs WHERE session_id = ?
             ORDER BY created_at DESC LIMIT ?",
        )?;
        let rows = stmt
            .query_map(params![session_id, limit], row_to_log_entry)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Database {
        Database::open_in_memory().expect("in-memory db")
    }

    fn create_user(db: &Database) -> i64 {
        let encrypted = encrypt("private_key", "password").expect("encrypt");
        db.create_user("user1", "ssh-ed25519 AAAA", &encrypted, "ed25519")
            .expect("create user")
    }

    #[test]
    fn settings_round_trip() {
        let db = setup_db();
        db.set_setting("mode", "development").expect("set");
        let value = db.get_setting("mode").expect("get");
        assert_eq!(value, Some("development".to_string()));
    }

    #[test]
    fn user_crud_and_encryption() {
        let db = setup_db();
        let user_id = create_user(&db);
        let user = db.get_user(user_id).expect("get").expect("exists");
        assert_eq!(user.username, "user1");
        assert_ne!(user.private_key_encrypted, "private_key");

        let users = db.list_users().expect("list");
        assert!(!users.is_empty());

        let deleted = db.delete_user(user_id).expect("delete");
        assert!(deleted);
    }

    #[test]
    fn user_limits_round_trip() {
        let db = setup_db();
        let user_id = create_user(&db);
        db.set_user_limits(user_id, Some(2), Some(1024), None)
            .expect("set limits");
        let limits = db.get_user_limits(user_id).expect("get").expect("exists");
        assert_eq!(limits.max_connections, Some(2));
        assert_eq!(limits.traffic_quota_bytes, Some(1024));
        let deleted = db.delete_user_limits(user_id).expect("delete");
        assert!(deleted);
    }

    #[test]
    fn traffic_usage_round_trip() {
        let db = setup_db();
        let user_id = create_user(&db);
        db.record_traffic_usage(user_id, 10, 20).expect("record");
        db.record_traffic_usage(user_id, 5, 5).expect("record");
        let rows = db.list_traffic_usage(user_id, 10).expect("list");
        assert_eq!(rows.len(), 2);
        let (up, down) = db.traffic_usage_totals(user_id).expect("sum");
        assert_eq!(up, 15);
        assert_eq!(down, 25);
        let cleared = db.clear_traffic_usage(user_id).expect("clear");
        assert!(cleared);
    }

    #[test]
    fn user_connections_round_trip() {
        let db = setup_db();
        let user_id = create_user(&db);
        let conn_id = db
            .add_user_connection(user_id, Some("127.0.0.1"))
            .expect("add");
        let active = db.list_active_connections(user_id).expect("list");
        assert_eq!(active.len(), 1);
        let count = db.count_active_connections(user_id).expect("count");
        assert_eq!(count, 1);
        let updated = db.set_connection_inactive(conn_id).expect("inactive");
        assert!(updated);
        let count_after = db.count_active_connections(user_id).expect("count");
        assert_eq!(count_after, 0);
        let cleared = db.clear_connections(user_id).expect("clear");
        assert!(cleared);
    }

    #[test]
    fn configs_round_trip() {
        let db = setup_db();
        let user_id = create_user(&db);
        let config_id = db
            .create_config(user_id, "ios", "default", "{\"log\":{}}")
            .expect("create");
        let config = db.get_config(config_id).expect("get").expect("exists");
        assert_eq!(config.platform, "ios");
        let list = db.list_configs_for_user(user_id).expect("list");
        assert_eq!(list.len(), 1);
        let deleted = db.delete_config(config_id).expect("delete");
        assert!(deleted);
    }

    #[test]
    fn sessions_and_logs_round_trip() {
        let db = setup_db();
        db.start_session("sess1").expect("start");
        let log_id = db
            .add_log(Some("sess1"), "INFO", Some("test"), "hello")
            .expect("log");
        assert!(log_id > 0);
        let logs = db.list_logs_for_session("sess1", 10).expect("list logs");
        assert_eq!(logs.len(), 1);
        let ended = db.end_session("sess1").expect("end");
        assert!(ended);
        let session = db.get_session("sess1").expect("get").expect("exists");
        assert!(session.ended_at.is_some());
    }
}
