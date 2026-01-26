// src/ssh/service.rs
// SSH service management
#![allow(dead_code)]

use crate::error::{AppError, Result};
use crate::utils::system::command_exists;
use std::process::Command;

pub fn restart_sshd() -> Result<()> {
    if command_exists("systemctl") {
        let output = Command::new("systemctl")
            .args(["restart", "ssh"])
            .output()
            .map_err(|e| AppError::System(e.to_string()))?;

        if output.status.success() {
            return Ok(());
        }

        let output = Command::new("systemctl")
            .args(["restart", "sshd"])
            .output()
            .map_err(|e| AppError::System(e.to_string()))?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::System(format!(
            "Failed to restart sshd: {}",
            stderr
        )));
    }

    if command_exists("service") {
        let output = Command::new("service")
            .args(["ssh", "restart"])
            .output()
            .map_err(|e| AppError::System(e.to_string()))?;

        if output.status.success() {
            return Ok(());
        }

        let output = Command::new("service")
            .args(["sshd", "restart"])
            .output()
            .map_err(|e| AppError::System(e.to_string()))?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::System(format!(
            "Failed to restart sshd: {}",
            stderr
        )));
    }

    Err(AppError::System(
        "No supported service manager found to restart sshd".to_string(),
    ))
}
