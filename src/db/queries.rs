// src/db/queries.rs
// Database queries

use super::{Database, User};
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

    pub fn get_user(&self, id: i64) -> Result<Option<User>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, username, public_key, private_key_encrypted, key_type, is_active, created_at, updated_at
             FROM users WHERE id = ?"
        )?;
        let result = stmt.query_row(params![id], row_to_user).optional()?;
        Ok(result)
    }

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
}
