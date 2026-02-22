//! Pacman infrastructure
//!
//! Handles package management via pacman.

use crate::error::{CommandErrorReturn, CommandOutput};
use std::process::Command;

type CommandResult<T> = std::result::Result<T, CommandErrorReturn>;

pub struct Pacman;

impl Pacman {
    /// Install packages
    pub fn install(packages: &[String]) -> CommandResult<CommandOutput> {
        Self::run_pacman(&["-S", "--noconfirm", "--needed"], packages)
    }

    /// Update database then install packages.
    /// This avoids stale mirror 404 errors.
    pub fn update_and_install(packages: &[String]) -> CommandResult<CommandOutput> {
        // First sync the database
        let _ = Self::update_db(); // best-effort, don't fail if this errors
        // Then install
        Self::install(packages)
    }

    /// Remove packages.
    ///
    /// - `recursive`: also remove unneeded dependencies (`-s`)
    /// - `force`: skip dependency checks entirely (`-dd`). Use with caution!
    pub fn remove(
        packages: &[String],
        recursive: bool,
        force: bool,
    ) -> CommandResult<CommandOutput> {
        let mut args = vec!["-R", "--noconfirm"];
        if force {
            args.push("-dd");
        }
        if recursive {
            args.push("-s");
        }
        Self::run_pacman(&args, packages)
    }

    /// Update database
    pub fn update_db() -> CommandResult<CommandOutput> {
        Self::run_pacman_raw(&["-Sy"])
    }

    /// Check if package is installed (read-only, no root needed)
    pub fn is_installed(package: &str) -> CommandResult<bool> {
        let output = Command::new("pacman")
            .arg("-Qi")
            .arg(package)
            .output()
            .map_err(|e| CommandErrorReturn {
                operation: format!("pacman -Qi {}", package),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        Ok(output.status.success())
    }

    /// Resolve a package name to its real installed name.
    /// Handles virtual packages (e.g. "netcat" → "gnu-netcat").
    /// Returns the original name if resolution fails.
    pub fn resolve_package(name: &str) -> String {
        match Command::new("pacman").arg("-Qq").arg(name).output() {
            Ok(o) if o.status.success() => {
                let resolved = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if resolved.is_empty() {
                    name.to_string()
                } else {
                    resolved.lines().next().unwrap_or(name).to_string()
                }
            }
            _ => name.to_string(),
        }
    }

    fn run_pacman(flags: &[&str], packages: &[String]) -> CommandResult<CommandOutput> {
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

    fn run_pacman_raw(args: &[&str]) -> CommandResult<CommandOutput> {
        let output =
            Command::new("pacman")
                .args(args)
                .output()
                .map_err(|e| CommandErrorReturn {
                    operation: format!("pacman {}", args.join(" ")),
                    exit_code: None,
                    stderr: e.to_string(),
                })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(CommandErrorReturn {
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
