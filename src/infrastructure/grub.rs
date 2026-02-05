//! GRUB infrastructure
//!
//! Handles GRUB configuration.

use crate::error::{CommandOutput, ModulariteaError, Result};
use std::fs;
use std::process::Command;

pub struct Grub;

impl Grub {
    pub const DEFAULT_CONFIG_PATH: &'static str = "/etc/default/grub";

    /// Set GRUB theme
    pub fn set_theme(theme_path: &str) -> Result<()> {
        Self::update_config("GRUB_THEME", theme_path)
    }

    /// Set timeout
    pub fn set_timeout(seconds: u32) -> Result<()> {
        Self::update_config("GRUB_TIMEOUT", &seconds.to_string())
    }

    /// Set default entry
    pub fn set_default(entry: &str) -> Result<()> {
        Self::update_config("GRUB_DEFAULT", entry)
    }

    /// Regenerate grub.cfg
    pub fn regenerate() -> Result<CommandOutput> {
        // Detect path (Arch/Debian usually rely on grub-mkconfig)
        let output = Command::new("grub-mkconfig")
            .arg("-o")
            .arg("/boot/grub/grub.cfg")
            .output()
            .map_err(|e| ModulariteaError::GrubError {
                operation: "grub-mkconfig".into(),
                reason: e.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(ModulariteaError::GrubError {
                operation: "grub-mkconfig".into(),
                reason: stderr,
            });
        }

        Ok(CommandOutput {
            exit_code: output.status.code().unwrap_or(0),
            stdout,
            stderr,
        })
    }

    /// Update a specific key in /etc/default/grub (Primitive regex/replacement)
    fn update_config(key: &str, value: &str) -> Result<()> {
        let content = fs::read_to_string(Self::DEFAULT_CONFIG_PATH).map_err(|e| {
            ModulariteaError::GrubError {
                operation: "read config".into(),
                reason: e.to_string(),
            }
        })?;

        let mut new_lines = Vec::new();
        let mut key_found = false;

        let quoted_value = format!("\"{}\"", value.replace("\"", "\\\""));

        for line in content.lines() {
            if line.trim().starts_with(&format!("{}=", key)) {
                new_lines.push(format!("{}={}", key, quoted_value));
                key_found = true;
            } else {
                new_lines.push(line.to_string());
            }
        }

        if !key_found {
            new_lines.push(format!("{}={}", key, quoted_value));
        }

        let new_content = new_lines.join("\n");
        fs::write(Self::DEFAULT_CONFIG_PATH, new_content).map_err(|e| {
            ModulariteaError::GrubError {
                operation: "write config".into(),
                reason: e.to_string(),
            }
        })?;

        Ok(())
    }
}
