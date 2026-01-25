// src/singbox/install.rs
// sing-box installation helpers
#![allow(dead_code)]

use crate::error::{AppError, Result};
use crate::utils::system;
use std::process::Command;

#[derive(Debug, Clone, Copy)]
enum PackageManager {
    Apt,
    Dnf,
    Yum,
    Apk,
    Pacman,
    Zypper,
}

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub display: String,
    pub command: String,
    pub args: Vec<String>,
}

impl CommandSpec {
    fn new(display: &str, command: &str, args: &[&str]) -> Self {
        Self {
            display: display.to_string(),
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }
}

fn detect_package_manager() -> Option<PackageManager> {
    if system::command_exists("apt-get") {
        Some(PackageManager::Apt)
    } else if system::command_exists("dnf") {
        Some(PackageManager::Dnf)
    } else if system::command_exists("yum") {
        Some(PackageManager::Yum)
    } else if system::command_exists("apk") {
        Some(PackageManager::Apk)
    } else if system::command_exists("pacman") {
        Some(PackageManager::Pacman)
    } else if system::command_exists("zypper") {
        Some(PackageManager::Zypper)
    } else {
        None
    }
}

fn ensure_root() -> Result<()> {
    if system::is_root() {
        Ok(())
    } else {
        Err(AppError::SingBox(
            "This action requires root privileges.".to_string(),
        ))
    }
}

fn run_command_capture(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!(
        "{}{}",
        stdout,
        if stderr.is_empty() { "" } else { "\n" }
    ) + &stderr;

    if output.status.success() {
        Ok(combined.trim().to_string())
    } else {
        Err(AppError::SingBox(combined.trim().to_string()))
    }
}

pub fn run_command_spec(spec: &CommandSpec) -> Result<String> {
    let args: Vec<&str> = spec.args.iter().map(|s| s.as_str()).collect();
    run_command_capture(&spec.command, &args)
}

pub fn install_steps() -> Result<Vec<CommandSpec>> {
    ensure_root()?;
    if system::command_exists("curl") {
        Ok(vec![CommandSpec::new(
            "curl -fsSL https://sing-box.app/deb-install.sh | bash",
            "bash",
            &["-c", "curl -fsSL https://sing-box.app/deb-install.sh | bash"],
        )])
    } else {
        match detect_package_manager() {
            Some(PackageManager::Apt) => Ok(vec![
                CommandSpec::new("apt-get update -y", "apt-get", &["update", "-y"]),
                CommandSpec::new(
                    "apt-get install -y sing-box",
                    "apt-get",
                    &["install", "-y", "sing-box"],
                ),
            ]),
            Some(PackageManager::Dnf) => Ok(vec![CommandSpec::new(
                "dnf install -y sing-box",
                "dnf",
                &["install", "-y", "sing-box"],
            )]),
            Some(PackageManager::Yum) => Ok(vec![CommandSpec::new(
                "yum install -y sing-box",
                "yum",
                &["install", "-y", "sing-box"],
            )]),
            Some(PackageManager::Apk) => Ok(vec![CommandSpec::new(
                "apk add sing-box",
                "apk",
                &["add", "sing-box"],
            )]),
            Some(PackageManager::Pacman) => Ok(vec![CommandSpec::new(
                "pacman -Sy --noconfirm sing-box",
                "pacman",
                &["-Sy", "--noconfirm", "sing-box"],
            )]),
            Some(PackageManager::Zypper) => Ok(vec![CommandSpec::new(
                "zypper --non-interactive install sing-box",
                "zypper",
                &["--non-interactive", "install", "sing-box"],
            )]),
            None => Err(AppError::SingBox(
                "No supported package manager found for installation.".to_string(),
            )),
        }
    }
}

pub fn reinstall_steps() -> Result<Vec<CommandSpec>> {
    install_steps()
}

pub fn uninstall_steps() -> Result<Vec<CommandSpec>> {
    ensure_root()?;
    match detect_package_manager() {
        Some(PackageManager::Apt) => Ok(vec![CommandSpec::new(
            "apt-get remove -y sing-box",
            "apt-get",
            &["remove", "-y", "sing-box"],
        )]),
        Some(PackageManager::Dnf) => Ok(vec![CommandSpec::new(
            "dnf remove -y sing-box",
            "dnf",
            &["remove", "-y", "sing-box"],
        )]),
        Some(PackageManager::Yum) => Ok(vec![CommandSpec::new(
            "yum remove -y sing-box",
            "yum",
            &["remove", "-y", "sing-box"],
        )]),
        Some(PackageManager::Apk) => Ok(vec![CommandSpec::new(
            "apk del sing-box",
            "apk",
            &["del", "sing-box"],
        )]),
        Some(PackageManager::Pacman) => Ok(vec![CommandSpec::new(
            "pacman -R --noconfirm sing-box",
            "pacman",
            &["-R", "--noconfirm", "sing-box"],
        )]),
        Some(PackageManager::Zypper) => Ok(vec![CommandSpec::new(
            "zypper --non-interactive remove sing-box",
            "zypper",
            &["--non-interactive", "remove", "sing-box"],
        )]),
        None => Err(AppError::SingBox(
            "No supported package manager found for uninstall.".to_string(),
        )),
    }
}

pub fn install_singbox() -> Result<String> {
    let steps = install_steps()?;
    let mut combined = String::new();
    for step in steps {
        let out = run_command_spec(&step)?;
        if !out.is_empty() {
            if !combined.is_empty() {
                combined.push('\n');
            }
            combined.push_str(&out);
        }
    }
    Ok(combined)
}

pub fn reinstall_singbox() -> Result<String> {
    install_singbox()
}

pub fn uninstall_singbox() -> Result<String> {
    let steps = uninstall_steps()?;
    let mut combined = String::new();
    for step in steps {
        let out = run_command_spec(&step)?;
        if !out.is_empty() {
            if !combined.is_empty() {
                combined.push('\n');
            }
            combined.push_str(&out);
        }
    }
    Ok(combined)
}
