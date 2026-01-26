// src/db/models.rs
// Database models
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// User model - represents an SSH proxy user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub public_key: String,
    pub private_key_encrypted: String,
    pub key_type: String,
    pub is_active: bool,
    pub created_at: String, // ISO 8601 string from SQLite
    pub updated_at: String, // ISO 8601 string from SQLite
}

/// User limits for connection/traffic restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLimits {
    pub user_id: i64,
    pub max_connections: Option<i64>,
    pub traffic_quota_bytes: Option<i64>,
    pub expires_at: Option<String>,
}

/// Traffic usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficUsage {
    pub id: i64,
    pub user_id: i64,
    pub bytes_up: i64,
    pub bytes_down: i64,
    pub recorded_at: String,
}

/// Generated client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub id: i64,
    pub user_id: i64,
    pub platform: String,
    pub routing_preset: String,
    pub config_json: String,
    pub created_at: String,
}

/// Application session for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: i64,
    pub session_id: Option<String>,
    pub level: String,
    pub target: Option<String>,
    pub message: String,
    pub created_at: String,
}

/// Active user connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConnection {
    pub id: i64,
    pub user_id: i64,
    pub connected_at: String,
    pub remote_ip: Option<String>,
    pub is_active: bool,
}
