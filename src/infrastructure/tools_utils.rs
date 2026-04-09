use crate::error::{CommandOutput, ModulariteaError, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct ToolsUtils;

pub struct PackageCacheCleaner;

pub mod mode {
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub enum SwapMode {
		Enable,
		Disable,
	}

	pub use SwapMode::Disable as disable;
	pub use SwapMode::Enable as enable;
}

pub struct Swap;

impl PackageCacheCleaner {
	pub fn clean() -> Result<CommandOutput> {
		let cmd = "rm -rf /var/cache/pacman/pkg/*";

		let output = Command::new("sh")
			.args(["-c", cmd])
			.output()
			.map_err(|e| ModulariteaError::CommandError {
				command: cmd.to_string(),
				exit_code: None,
				stderr: e.to_string(),
			})?;

		let stdout = String::from_utf8_lossy(&output.stdout).to_string();
		let stderr = String::from_utf8_lossy(&output.stderr).to_string();

		if !output.status.success() {
			return Err(ModulariteaError::CommandError {
				command: cmd.to_string(),
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

	/// Helper to quickly check if cache cleaning succeeded.
	///
	/// Returns:
	/// - `true`  => clear cache success
	/// - `false` => clear cache failed
	pub fn try_clean_and_check() -> bool {
		Self::clean().is_ok()
	}

	
}

impl Swap {
	const ZRAM_CONF_PATH: &'static str = "/etc/systemd/zram-generator.conf";
    const ZRAM_CONF_CONTENT: &'static str = "[zram0]\nzram-size = ram / 3\ncompression-algorithm = zstd\nswap-priority = 100\n";

	/// API usage:
	/// `Swap::set(mode::enable)` or `Swap::set(mode::disable)`.
	pub fn set(mode: mode::SwapMode) -> Result<CommandOutput> {
		match mode {
			mode::SwapMode::Enable => Self::enable(),
			mode::SwapMode::Disable => Self::disable(),
		}
	}

	fn enable() -> Result<CommandOutput> {
		fs::write(Self::ZRAM_CONF_PATH, Self::ZRAM_CONF_CONTENT).map_err(|e| {
			ModulariteaError::FilesystemError {
				operation: format!("write {}", Self::ZRAM_CONF_PATH),
				source: e,
			}
		})?;

		let mut logs = String::new();

		let reload = Self::run_command("systemctl", &["daemon-reload"])?;
		logs.push_str(&reload.stdout);

		let service_result = Self::run_command("systemctl", &["restart", "systemd-zram-setup@zram0.service"])
			.or_else(|_| Self::run_command("systemctl", &["start", "systemd-zram-setup@zram0.service"]))?;
		logs.push_str(&service_result.stdout);

		Ok(CommandOutput {
			exit_code: 0,
			stdout: if logs.trim().is_empty() {
				"ZRAM swap enabled (persistent, size = RAM/3, zstd)".to_string()
			} else {
				logs
			},
			stderr: String::new(),
		})
	}

	fn disable() -> Result<CommandOutput> {
		let mut logs = String::new();

		let _ = Self::run_command("systemctl", &["stop", "systemd-zram-setup@zram0.service"]);
		let _ = Self::run_command("swapoff", &["/dev/zram0"]);

		if Path::new(Self::ZRAM_CONF_PATH).exists() {
			fs::remove_file(Self::ZRAM_CONF_PATH).map_err(|e| ModulariteaError::FilesystemError {
				operation: format!("remove {}", Self::ZRAM_CONF_PATH),
				source: e,
			})?;
		}

		let reload = Self::run_command("systemctl", &["daemon-reload"])?;
		logs.push_str(&reload.stdout);

		Ok(CommandOutput {
			exit_code: 0,
			stdout: if logs.trim().is_empty() {
				"ZRAM swap disabled (persistent)".to_string()
			} else {
				logs
			},
			stderr: String::new(),
		})
	}

	fn run_command(bin: &str, args: &[&str]) -> Result<CommandOutput> {
		let output = Command::new(bin)
			.args(args)
			.output()
			.map_err(|e| ModulariteaError::CommandError {
				command: format!("{} {}", bin, args.join(" ")),
				exit_code: None,
				stderr: e.to_string(),
			})?;

		let stdout = String::from_utf8_lossy(&output.stdout).to_string();
		let stderr = String::from_utf8_lossy(&output.stderr).to_string();

		if !output.status.success() {
			return Err(ModulariteaError::CommandError {
				command: format!("{} {}", bin, args.join(" ")),
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
