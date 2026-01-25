// src/ssh/ports.rs
// SSH port detection and configuration
#![allow(dead_code)]

use crate::error::{AppError, Result};
use crate::utils::system::is_port_available;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_SSHD_CONFIG: &str = "/etc/ssh/sshd_config";
const MIN_PROXY_PORT: u16 = 1024;
const MAX_PROXY_PORT: u16 = 60000;

pub fn sshd_config_path() -> PathBuf {
    PathBuf::from(DEFAULT_SSHD_CONFIG)
}

pub fn list_ports(path: &Path) -> Result<Vec<u16>> {
    let contents = fs::read_to_string(path).map_err(AppError::Io)?;
    Ok(parse_ports(&contents))
}

pub fn validate_custom_port(port: u16, existing_ports: &[u16]) -> Result<()> {
    if port == 22 {
        return Err(AppError::Config(
            "Custom SSH port must not be 22".to_string(),
        ));
    }

    if port < MIN_PROXY_PORT || port > MAX_PROXY_PORT {
        return Err(AppError::Config(format!(
            "Custom SSH port must be between {} and {}",
            MIN_PROXY_PORT, MAX_PROXY_PORT
        )));
    }

    if existing_ports.iter().any(|existing| *existing == port) {
        return Err(AppError::Config(format!(
            "Custom SSH port {} already configured",
            port
        )));
    }

    if !is_port_available(port) {
        return Err(AppError::Config(format!(
            "Custom SSH port {} is already in use",
            port
        )));
    }

    Ok(())
}

pub fn add_port(path: &Path, port: u16) -> Result<bool> {
    let contents = fs::read_to_string(path).map_err(AppError::Io)?;
    let (updated, changed) = apply_port_update(&contents, port, true)?;
    if changed {
        fs::write(path, updated).map_err(AppError::Io)?;
    }
    Ok(changed)
}

fn parse_ports(contents: &str) -> Vec<u16> {
    let mut ports = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let directive = match parts.next() {
            Some(value) => value,
            None => continue,
        };
        if !directive.eq_ignore_ascii_case("Port") {
            continue;
        }
        if let Some(port_str) = parts.next() {
            if let Ok(port) = port_str.parse::<u16>() {
                if !ports.contains(&port) {
                    ports.push(port);
                }
            }
        }
    }
    ports
}

fn apply_port_update(contents: &str, port: u16, check_available: bool) -> Result<(String, bool)> {
    let existing_ports = parse_ports(contents);
    if existing_ports.iter().any(|existing| *existing == port) {
        return Ok((contents.to_string(), false));
    }

    if port == 22 {
        return Err(AppError::Config(
            "Custom SSH port must not be 22".to_string(),
        ));
    }

    if port < MIN_PROXY_PORT || port > MAX_PROXY_PORT {
        return Err(AppError::Config(format!(
            "Custom SSH port must be between {} and {}",
            MIN_PROXY_PORT, MAX_PROXY_PORT
        )));
    }

    if check_available && !is_port_available(port) {
        return Err(AppError::Config(format!(
            "Custom SSH port {} is already in use",
            port
        )));
    }

    let mut lines: Vec<String> = contents.lines().map(|line| line.to_string()).collect();
    let insert_index = lines
        .iter()
        .position(|line| line.trim_start().starts_with("Match "));

    let port_line = format!("Port {}", port);
    match insert_index {
        Some(index) => lines.insert(index, port_line),
        None => lines.push(port_line),
    }

    let mut output = lines.join("\n");
    output.push('\n');

    Ok((output, true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ports_collects_unique_entries() {
        let contents = r#"
Port 22
Port 7344
Port 22
# Port 9999
"#;
        let ports = parse_ports(contents);
        assert_eq!(ports, vec![22, 7344]);
    }

    #[test]
    fn add_port_inserts_before_match_block() {
        let contents = "Port 22\nMatch User root\n  X11Forwarding no\n";
        let (updated, changed) = apply_port_update(contents, 7344, false).unwrap();
        assert!(changed);
        let expected = "Port 22\nPort 7344\nMatch User root\n  X11Forwarding no\n";
        assert_eq!(updated, expected);
    }

    #[test]
    fn add_port_is_noop_when_existing() {
        let contents = "Port 22\nPort 7344\n";
        let (updated, changed) = apply_port_update(contents, 7344, false).unwrap();
        assert!(!changed);
        assert_eq!(updated, contents);
    }
}
