// src/singbox/service.rs
// sing-box systemd service management
#![allow(dead_code)]

use crate::error::{AppError, Result};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    NotFound,
    Unknown,
}

/// Get the status of sing-box service
pub fn get_service_status() -> Result<ServiceStatus> {
    let output = Command::new("systemctl")
        .args(["is-active", "sing-box"])
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    let status = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(match status.as_str() {
        "active" => ServiceStatus::Running,
        "inactive" => ServiceStatus::Stopped,
        "failed" => ServiceStatus::Stopped,
        _ => {
            // Check if service exists
            let exists = Command::new("systemctl")
                .args(["cat", "sing-box"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if exists {
                ServiceStatus::Unknown
            } else {
                ServiceStatus::NotFound
            }
        }
    })
}

/// Start sing-box service
pub fn start_service() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["start", "sing-box"])
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::SingBox(format!(
            "Failed to start service: {}",
            stderr
        )));
    }

    Ok(())
}

/// Stop sing-box service
pub fn stop_service() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["stop", "sing-box"])
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::SingBox(format!(
            "Failed to stop service: {}",
            stderr
        )));
    }

    Ok(())
}

/// Restart sing-box service
pub fn restart_service() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["restart", "sing-box"])
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::SingBox(format!(
            "Failed to restart service: {}",
            stderr
        )));
    }

    Ok(())
}

/// Enable sing-box service
pub fn enable_service() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["enable", "--now", "sing-box"])
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::SingBox(format!(
            "Failed to enable service: {}",
            stderr
        )));
    }

    Ok(())
}

/// Disable sing-box service
pub fn disable_service() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["disable", "--now", "sing-box"])
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::SingBox(format!(
            "Failed to disable service: {}",
            stderr
        )));
    }

    Ok(())
}

/// Reload sing-box configuration
pub fn reload_service() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["reload", "sing-box"])
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    if !output.status.success() {
        // Try restart if reload fails
        restart_service()?;
    }

    Ok(())
}

pub fn get_service_status_output(lines: usize) -> Result<String> {
    let output = Command::new("systemctl")
        .args([
            "status",
            "sing-box",
            "--no-pager",
            &format!("--lines={}", lines),
        ])
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else if !output.stderr.is_empty() {
        Ok(String::from_utf8_lossy(&output.stderr).trim().to_string())
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
