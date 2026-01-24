// src/ssh/users.rs
// Linux system user management

use crate::error::{AppError, Result};
use std::process::Command;

/// Validate username according to Linux conventions
pub fn validate_username(username: &str) -> Result<()> {
    if username.is_empty() {
        return Err(AppError::User("Username cannot be empty".to_string()));
    }

    if username.len() > 32 {
        return Err(AppError::User(
            "Username must be 32 characters or less".to_string(),
        ));
    }

    // Must start with lowercase letter or underscore
    let first_char = username.chars().next().unwrap();
    if !first_char.is_ascii_lowercase() && first_char != '_' {
        return Err(AppError::User(
            "Username must start with a lowercase letter or underscore".to_string(),
        ));
    }

    // Can only contain lowercase letters, digits, underscores, and hyphens
    for c in username.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_' && c != '-' {
            return Err(AppError::User(format!(
                "Username contains invalid character: {}",
                c
            )));
        }
    }

    // Cannot be a reserved name
    let reserved = [
        "root",
        "admin",
        "daemon",
        "bin",
        "sys",
        "sync",
        "games",
        "man",
        "mail",
        "news",
        "uucp",
        "proxy",
        "www-data",
        "backup",
        "list",
        "irc",
        "gnats",
        "nobody",
        "systemd-network",
        "systemd-resolve",
        "syslog",
        "messagebus",
        "uuidd",
        "sshd",
    ];
    if reserved.contains(&username) {
        return Err(AppError::User(format!(
            "Username '{}' is reserved",
            username
        )));
    }

    Ok(())
}

/// Check if a system user exists
pub fn user_exists(username: &str) -> Result<bool> {
    let output = Command::new("id")
        .arg(username)
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    Ok(output.status.success())
}

/// Create a system user with /sbin/nologin shell
pub fn create_system_user(username: &str) -> Result<()> {
    validate_username(username)?;

    if user_exists(username)? {
        return Err(AppError::User(format!(
            "User '{}' already exists",
            username
        )));
    }

    let output = Command::new("useradd")
        .args([
            "--create-home",
            "--shell",
            "/sbin/nologin",
            "--comment",
            "sbconfig proxy user",
            username,
        ])
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::System(format!(
            "Failed to create user: {}",
            stderr
        )));
    }

    Ok(())
}

/// Delete a system user and their home directory
pub fn delete_system_user(username: &str) -> Result<()> {
    if !user_exists(username)? {
        return Err(AppError::User(format!(
            "User '{}' does not exist",
            username
        )));
    }

    let output = Command::new("userdel")
        .args(["--remove", username])
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::System(format!(
            "Failed to delete user: {}",
            stderr
        )));
    }

    Ok(())
}

/// Setup authorized_keys for a user
pub fn setup_authorized_keys(username: &str, public_key: &str) -> Result<()> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    // Get user's home directory
    let output = Command::new("getent")
        .args(["passwd", username])
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    if !output.status.success() {
        return Err(AppError::User(format!("User '{}' not found", username)));
    }

    let passwd_line = String::from_utf8_lossy(&output.stdout);
    let home_dir = passwd_line
        .split(':')
        .nth(5)
        .ok_or_else(|| AppError::System("Failed to parse home directory".to_string()))?
        .trim();

    let ssh_dir = format!("{}/.ssh", home_dir);
    let auth_keys_path = format!("{}/authorized_keys", ssh_dir);

    // Create .ssh directory
    fs::create_dir_all(&ssh_dir).map_err(|e| AppError::Io(e))?;

    // Write authorized_keys
    fs::write(&auth_keys_path, format!("{}\n", public_key)).map_err(|e| AppError::Io(e))?;

    // Set permissions
    fs::set_permissions(&ssh_dir, fs::Permissions::from_mode(0o700))
        .map_err(|e| AppError::Io(e))?;
    fs::set_permissions(&auth_keys_path, fs::Permissions::from_mode(0o600))
        .map_err(|e| AppError::Io(e))?;

    // Change ownership to the user
    let output = Command::new("chown")
        .args(["-R", &format!("{}:{}", username, username), &ssh_dir])
        .output()
        .map_err(|e| AppError::System(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::System(format!(
            "Failed to set ownership: {}",
            stderr
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username_valid() {
        assert!(validate_username("john").is_ok());
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("proxy_user").is_ok());
        assert!(validate_username("test-user").is_ok());
        assert!(validate_username("_private").is_ok());
    }

    #[test]
    fn test_validate_username_invalid() {
        assert!(validate_username("").is_err());
        assert!(validate_username("Root").is_err()); // uppercase
        assert!(validate_username("123user").is_err()); // starts with number
        assert!(validate_username("user@name").is_err()); // invalid char
        assert!(validate_username("root").is_err()); // reserved
        assert!(validate_username("a".repeat(33).as_str()).is_err()); // too long
    }
}
