// src/utils/system.rs
// System utility functions

use crate::error::{AppError, Result};
use std::process::Command;

/// Check if running as root
pub fn is_root() -> bool {
    // Use the id command which is portable across Unix systems
    Command::new("id")
        .args(["-u"])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<u32>()
                .map(|uid| uid == 0)
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

/// Get the hostname
pub fn get_hostname() -> Result<String> {
    let output = Command::new("hostname")
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get public IP address (using external service)
pub fn get_public_ip() -> Result<String> {
    let output = Command::new("curl")
        .args(["-s", "-4", "ifconfig.me"])
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(AppError::System("Failed to get public IP".to_string()))
    }
}

/// Check if a port is available
pub fn is_port_available(port: u16) -> bool {
    use std::net::TcpListener;
    TcpListener::bind(("0.0.0.0", port)).is_ok()
}

/// Generate a random available port in a range
pub fn find_available_port(start: u16, end: u16) -> Option<u16> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        let port = rng.gen_range(start..=end);
        if is_port_available(port) {
            return Some(port);
        }
    }
    None
}
