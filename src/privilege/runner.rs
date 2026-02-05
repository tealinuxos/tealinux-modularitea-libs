//! Privilege Runner
//!
//! Executes commands using pkexec.

use crate::error::{CommandOutput, ModulariteaError, Result};
use std::process::Command;
use tracing::debug;

pub struct PrivilegeRunner;

impl PrivilegeRunner {
    /// Execute a command with pkexec
    pub fn run(binary: &str, args: &[&str]) -> Result<CommandOutput> {
        // TODO: Check if pkexec is available
        // TODO: Check if binary exists (maybe resolve path)

        let mut cmd = Command::new("pkexec");

        // If the binary is one of our own, we might need to find its path
        // For now, assume it's in PATH or absolute.
        // In development, it might be in target/debug/ or target/release/
        let binary_path = Self::resolve_binary(binary)?;

        cmd.arg(&binary_path);
        cmd.args(args);

        debug!(
            "Executing privileged command: pkexec {} {}",
            binary,
            args.join(" ")
        );

        let output = cmd.output().map_err(|e| ModulariteaError::CommandError {
            command: format!("pkexec {} {}", binary, args.join(" ")),
            exit_code: None,
            stderr: e.to_string(),
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let exit_code = output.status.code().unwrap_or(-1);

        if !output.status.success() {
            // Handle cancellation specifically (exit code 126 or 127 sometimes, usually 126 for auth failed or cancelled)
            // Polkit often returns 126 or 127.
            // 127: command not found
            // 126: command invoked cannot execute (auth failure)
            if exit_code == 126 || stderr.contains("dismissed") {
                return Err(ModulariteaError::PolkitCancelled);
            }

            return Err(ModulariteaError::CommandError {
                command: format!("{} {}", binary, args.join(" ")),
                exit_code: Some(exit_code),
                stderr,
            });
        }

        Ok(CommandOutput {
            exit_code,
            stdout,
            stderr,
        })
    }

    /// Run a known modularitea root binary
    pub fn run_modularitea(tool: &str, action: &str, args: &[&str]) -> Result<CommandOutput> {
        let binary = format!("modularitea-{}", tool);
        let mut full_args = vec![action];
        full_args.extend_from_slice(args);

        Self::run(&binary, &full_args)
    }

    fn resolve_binary(name: &str) -> Result<String> {
        // If absolute path, return as is
        if name.starts_with('/') {
            return Ok(name.to_string());
        }

        // Try to find in current executable directory (useful for dev/portable)
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                let candidate = parent.join(name);
                if candidate.exists() {
                    return Ok(candidate.to_string_lossy().to_string());
                }
            }
        }

        // Fallback to expecting it in PATH
        // We could use `which` crate here but for now just pass the name
        // and let pkexec handle path resolution if configured,
        // ALTHOUGH pkexec often requires absolute paths.
        // It's safer to return the name and let the system resolve,
        // OR we can explicitly checks `which` implementation.

        // For development, assuming current dir as well
        if std::path::Path::new(name).exists() {
            return Ok(format!("./{}", name));
        }

        Ok(name.to_string())
    }
}
