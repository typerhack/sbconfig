// src/error.rs
// Application error types

use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("SSH key error: {0}")]
    SshKey(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Setup not complete: {0}")]
    SetupIncomplete(String),

    #[error("User error: {0}")]
    User(String),

    #[error("sing-box error: {0}")]
    SingBox(String),

    #[error("System error: {0}")]
    System(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
