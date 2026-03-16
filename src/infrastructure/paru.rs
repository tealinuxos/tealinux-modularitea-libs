//! Paru AUR Helper infrastructure
//!
//! Wraps the `paru` CLI for AUR package operations.
//! Paru runs as the current user (not root) and handles
//! privilege escalation internally via sudo when needed.

use crate::domain::aur_package::InstalledAurPackage;
use crate::error::{CommandErrorReturn, CommandOutput};
use std::process::Command;

type CommandResult<T> = std::result::Result<T, CommandErrorReturn>;

/// Name validation regex pattern for package names
const VALID_PKG_NAME: &str = r"^[a-zA-Z0-9@._+\-]+$";

pub struct Paru;

impl Paru {
    /// Check if paru is available on the system
    pub fn is_available() -> bool {
        Command::new("which")
            .arg("paru")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Validate a package name to prevent command injection
    fn validate_package_name(name: &str) -> CommandResult<()> {
        if name.is_empty() {
            return Err(CommandErrorReturn {
                operation: "validate_package_name".to_string(),
                exit_code: Some(1),
                stderr: "Package name cannot be empty".to_string(),
            });
        }

        // Simple validation: only allow alphanumeric, @, ., _, +, -
        let valid = name
            .chars()
            .all(|c| c.is_alphanumeric() || "@._+-".contains(c));

        if !valid {
            return Err(CommandErrorReturn {
                operation: "validate_package_name".to_string(),
                exit_code: Some(1),
                stderr: format!(
                    "Invalid package name '{}': must match {}",
                    name, VALID_PKG_NAME
                ),
            });
        }

        Ok(())
    }

    /// Install an AUR package via paru
    ///
    /// Runs: `paru -S --noconfirm --needed <package>`
    pub fn install(package: &str) -> CommandResult<CommandOutput> {
        Self::validate_package_name(package)?;
        Self::run_paru(&["-S", "--noconfirm", "--needed", package])
    }

    /// Remove an AUR package via paru
    ///
    /// Runs: `paru -Rns --noconfirm <package>`
    pub fn remove(package: &str) -> CommandResult<CommandOutput> {
        Self::validate_package_name(package)?;
        Self::run_paru(&["-Rns", "--noconfirm", package])
    }

    /// Update all AUR packages
    ///
    /// Runs: `paru -Sua --noconfirm`
    pub fn update() -> CommandResult<CommandOutput> {
        Self::run_paru(&["-Sua", "--noconfirm"])
    }

    /// List all installed foreign (AUR) packages
    ///
    /// Runs: `paru -Qm` and parses output
    pub fn list_installed() -> CommandResult<Vec<InstalledAurPackage>> {
        let output = Self::run_paru(&["-Qm"])?;

        let packages = output
            .stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    Some(InstalledAurPackage {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    /// Check if a specific package is installed
    pub fn is_installed(package: &str) -> CommandResult<bool> {
        let output = Command::new("pacman")
            .args(["-Qi", package])
            .output()
            .map_err(|e| CommandErrorReturn {
                operation: format!("pacman -Qi {}", package),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        Ok(output.status.success())
    }

    /// Run a paru command and return the output
    fn run_paru(args: &[&str]) -> CommandResult<CommandOutput> {
        let output = Command::new("paru")
            .args(args)
            .output()
            .map_err(|e| CommandErrorReturn {
                operation: format!("paru {}", args.join(" ")),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(CommandErrorReturn {
                operation: format!("paru {}", args.join(" ")),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_names() {
        assert!(Paru::validate_package_name("paru").is_ok());
        assert!(Paru::validate_package_name("visual-studio-code-bin").is_ok());
        assert!(Paru::validate_package_name("python-pip").is_ok());
        assert!(Paru::validate_package_name("lib32-mesa").is_ok());
        assert!(Paru::validate_package_name("r8168-dkms").is_ok());
        assert!(Paru::validate_package_name("ttf-ms-win11-auto").is_ok());
    }

    #[test]
    fn test_validate_invalid_names() {
        assert!(Paru::validate_package_name("").is_err());
        assert!(Paru::validate_package_name("pkg; rm -rf /").is_err());
        assert!(Paru::validate_package_name("pkg && echo").is_err());
        assert!(Paru::validate_package_name("$(whoami)").is_err());
        assert!(Paru::validate_package_name("pkg`id`").is_err());
    }
}
