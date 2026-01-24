// src/singbox/detect.rs
// sing-box installation detection

use crate::error::{AppError, Result};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SingBoxInfo {
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

/// Detect if sing-box is installed and get its version
pub fn detect_singbox() -> Result<SingBoxInfo> {
    // Try to find sing-box
    let which_output = Command::new("which")
        .arg("sing-box")
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    if !which_output.status.success() {
        return Ok(SingBoxInfo {
            installed: false,
            path: None,
            version: None,
        });
    }

    let path = String::from_utf8_lossy(&which_output.stdout)
        .trim()
        .to_string();

    // Get version
    let version_output = Command::new("sing-box")
        .arg("version")
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    let version = if version_output.status.success() {
        let output = String::from_utf8_lossy(&version_output.stdout);
        // Parse "sing-box version 1.x.x"
        output
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("sing-box version "))
            .map(|v| v.trim().to_string())
    } else {
        None
    };

    Ok(SingBoxInfo {
        installed: true,
        path: Some(path),
        version,
    })
}

/// Get installation instructions for sing-box
pub fn get_install_instructions() -> &'static str {
    r#"sing-box is not installed. Please install it manually:

Official installation guide:
  https://sing-box.sagernet.org/installation/package-manager/

For Debian/Ubuntu:
  curl -fsSL https://sing-box.app/deb-install.sh | sudo bash

For other systems, visit the official documentation."#
}
