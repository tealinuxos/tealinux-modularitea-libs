//! Pacman infrastructure
//!
//! Handles package management via pacman.

use crate::error::{CommandOutput, ModulariteaError, Result};
use std::process::Command;

pub struct Pacman;

impl Pacman {
    /// Install packages
    pub fn install(packages: &[String]) -> Result<CommandOutput> {
        Self::run_pacman(&["-S", "--noconfirm", "--needed"], packages)
    }

    /// Remove packages
    pub fn remove(packages: &[String], recursive: bool) -> Result<CommandOutput> {
        let mut args = vec!["-R", "--noconfirm"];
        if recursive {
            args.push("-s");
        }
        Self::run_pacman(&args, packages)
    }

    /// Update database
    pub fn update_db() -> Result<CommandOutput> {
        Self::run_pacman_raw(&["-Sy"])
    }

    /// Check if package is installed (read-only)
    pub fn is_installed(package: &str) -> Result<bool> {
        let output = Command::new("pacman")
            .arg("-Qi")
            .arg(package)
            .output()
            .map_err(|e| ModulariteaError::CommandError {
                command: format!("pacman -Qi {}", package),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        Ok(output.status.success())
    }

    fn run_pacman(flags: &[&str], packages: &[String]) -> Result<CommandOutput> {
        if packages.is_empty() {
            return Ok(CommandOutput {
                exit_code: 0,
                stdout: "".into(),
                stderr: "".into(),
            });
        }

        let mut args = flags.to_vec();
        for pkg in packages {
            args.push(pkg);
        }

        Self::run_pacman_raw(&args)
    }

    fn run_pacman_raw(args: &[&str]) -> Result<CommandOutput> {
        let output = Command::new("pacman").args(args).output().map_err(|e| {
            ModulariteaError::PacmanError {
                operation: format!("pacman {}", args.join(" ")),
                exit_code: None,
                stderr: e.to_string(),
            }
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(ModulariteaError::PacmanError {
                operation: format!("pacman {}", args.join(" ")),
                exit_code: output.status.code(),
                stderr,
            });
        }

        Ok(CommandOutput {
            exit_code: output.status.code().unwrap_or(0),
            stdout,
            stderr,
        })
    }
}
