// src/db/database.rs
// Database connection and initialization

use crate::error::Result;
use super::migrations;
use rusqlite::Connection;
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create database at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let mut db = Self { conn };
        db.init()?;
        Ok(db)
    }

    /// Open in-memory database (for testing)
    #[allow(dead_code)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let mut db = Self { conn };
        db.init()?;
        Ok(db)
    }

    /// Initialize database schema
    fn init(&mut self) -> Result<()> {
        migrations::run_migrations(&mut self.conn)
    }

    /// Get a reference to the connection
    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
