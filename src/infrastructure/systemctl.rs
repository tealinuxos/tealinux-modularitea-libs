//! Systemctl infrastructure
//!
//! Handles systemd services.

use crate::error::{CommandOutput, ModulariteaError, Result};
use std::process::Command;

pub struct Systemctl;

impl Systemctl {
    pub fn enable(services: &[String], now: bool, user: bool) -> Result<CommandOutput> {
        let mut args = vec!["enable"];
        if now {
            args.push("--now");
        }
        Self::run_systemctl(&args, services, user)
    }

    pub fn disable(services: &[String], now: bool, user: bool) -> Result<CommandOutput> {
        let mut args = vec!["disable"];
        if now {
            args.push("--now");
        }
        Self::run_systemctl(&args, services, user)
    }

    pub fn start(services: &[String], user: bool) -> Result<CommandOutput> {
        Self::run_systemctl(&["start"][..], services, user)
    }

    pub fn stop(services: &[String], user: bool) -> Result<CommandOutput> {
        Self::run_systemctl(&["stop"][..], services, user)
    }

    pub fn restart(services: &[String], user: bool) -> Result<CommandOutput> {
        Self::run_systemctl(&["restart"][..], services, user)
    }

    pub fn mask(services: &[String]) -> Result<CommandOutput> {
        Self::run_systemctl(&["mask"][..], services, false)
    }

    pub fn is_active(service: &str, user: bool) -> Result<bool> {
        let mut cmd = Command::new("systemctl");
        if user {
            cmd.arg("--user");
        }
        let output = cmd.arg("is-active").arg(service).output().map_err(|e| {
            ModulariteaError::SystemctlError {
                operation: format!("is-active {}", service),
                exit_code: None,
                stderr: e.to_string(),
            }
        })?;

        Ok(output.status.success())
    }

    fn run_systemctl(action: &[&str], services: &[String], user: bool) -> Result<CommandOutput> {
        if services.is_empty() {
            return Ok(CommandOutput {
                exit_code: 0,
                stdout: "".into(),
                stderr: "".into(),
            });
        }

        let mut args = Vec::new();
        if user {
            args.push("--user");
        }
        args.extend_from_slice(action);

        for svc in services {
            args.push(svc);
        }

        Self::run_systemctl_raw(&args)
    }

    fn run_systemctl_raw(args: &[&str]) -> Result<CommandOutput> {
        let output = Command::new("systemctl").args(args).output().map_err(|e| {
            ModulariteaError::SystemctlError {
                operation: format!("systemctl {}", args.join(" ")),
                exit_code: None,
                stderr: e.to_string(),
            }
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            // Systemctl errors often go to stderr
            return Err(ModulariteaError::SystemctlError {
                operation: format!("systemctl {}", args.join(" ")),
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
