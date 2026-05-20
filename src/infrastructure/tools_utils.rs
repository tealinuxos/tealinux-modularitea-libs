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
		// let cmd = "/usr/bin/pkexec  -rf ";

		// println!("executing: {}", cmd);

		let unnormalized_cmd = "pkexec /usr/bin/rm -rf /var/cache/pacman/pkg/*";

		let output = duct::cmd!("sh", "-c", unnormalized_cmd)
			.stdout_capture()
			.stderr_capture()
			.run()
			.map_err(|e| ModulariteaError::CommandError {
				command: unnormalized_cmd.to_string(),
				exit_code: None,
				stderr: e.to_string(),
			})?;

		let stdout = String::from_utf8_lossy(&output.stdout).to_string();
		let stderr = String::from_utf8_lossy(&output.stderr).to_string();

		if !output.status.success() {

			let sysret = ModulariteaError::CommandError {
				command: unnormalized_cmd.to_string(),
				exit_code: output.status.code(),
				stderr,
			};

			println!("{:?}", sysret);

			return Err(sysret);
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

		let reload_output = duct::cmd!("systemctl", "daemon-reload")
			.stdout_capture()
			.stderr_capture()
			.run()
			.map_err(|e| ModulariteaError::CommandError {
				command: "systemctl daemon-reload".to_string(),
				exit_code: None,
				stderr: e.to_string(),
			})?;
		let reload = Self::output_to_command_output("systemctl daemon-reload", reload_output)?;
		logs.push_str(&reload.stdout);

		let restart_output = duct::cmd!("systemctl", "restart", "systemd-zram-setup@zram0.service")
			.stdout_capture()
			.stderr_capture()
			.run()
			.map_err(|e| ModulariteaError::CommandError {
				command: "systemctl restart systemd-zram-setup@zram0.service".to_string(),
				exit_code: None,
				stderr: e.to_string(),
			})?;

		let (service_cmd, service_output) = if restart_output.status.success() {
			("systemctl restart systemd-zram-setup@zram0.service", restart_output)
		} else {
			let start_output = duct::cmd!("systemctl", "start", "systemd-zram-setup@zram0.service")
				.stdout_capture()
				.stderr_capture()
				.run()
				.map_err(|e| ModulariteaError::CommandError {
					command: "systemctl start systemd-zram-setup@zram0.service".to_string(),
					exit_code: None,
					stderr: e.to_string(),
				})?;
			("systemctl start systemd-zram-setup@zram0.service", start_output)
		};

		let service_result = Self::output_to_command_output(service_cmd, service_output)?;
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
		let output = duct::cmd(bin, args)
			.stdout_capture()
			.stderr_capture()
			.run()
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

	fn output_to_command_output(command: &str, output: std::process::Output) -> Result<CommandOutput> {
		let stdout = String::from_utf8_lossy(&output.stdout).to_string();
		let stderr = String::from_utf8_lossy(&output.stderr).to_string();

		if !output.status.success() {
			return Err(ModulariteaError::CommandError {
				command: command.to_string(),
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

pub struct MirrorUtils {
	pub country: Option<String>
}

impl MirrorUtils {
	pub fn set_country(country: Option<String>) -> Self {
		Self { country }
	}

	pub fn refresh_fastest_mirror(&self) -> Result<CommandOutput> {
		let country = self.country.as_deref().unwrap_or("Indonesia");
		let cmd = format!("pkexec reflector --country {} --save /etc/pacman.d/mirrorlist --verbose", country);

		let output = Command::new("sh")
			.args(["-c", &cmd])
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

	// pub fn try_refresh_fastest_mirror() -> bool {
	// 	// Self::refresh_fastest_mirror().is_ok()
	// }
}

pub struct DnsSwitcher;

impl DnsSwitcher {
	const RESOLV_CONF_PATH: &'static str = "/etc/resolv.conf";

	pub fn switch(provider: &str) -> Result<CommandOutput> {
		let provider_key = provider.trim().to_lowercase();
		let nameservers = Self::provider_nameservers(&provider_key).ok_or_else(|| {
			ModulariteaError::CommandError {
				command: format!("dns-switch {}", provider),
				exit_code: None,
				stderr: format!("Unsupported DNS provider: {}", provider),
			}
		})?;

		let _ = Self::run_command("chattr", &["-i", Self::RESOLV_CONF_PATH]);

		let resolv_content = nameservers
			.iter()
			.map(|ip| format!("nameserver {}\n", ip))
			.collect::<String>();

		fs::write(Self::RESOLV_CONF_PATH, resolv_content).map_err(|e| {
			ModulariteaError::FilesystemError {
				operation: format!("write {}", Self::RESOLV_CONF_PATH),
				source: e,
			}
		})?;

		let lock_result = Self::run_command("chattr", &["+i", Self::RESOLV_CONF_PATH])?;

		Ok(CommandOutput {
			exit_code: 0,
			stdout: format!("DNS switched to '{}'", provider_key),
			stderr: lock_result.stderr,
		})
	}

	pub fn try_switch_and_check(provider: &str) -> bool {
		Self::switch(provider).is_ok()
	}

	fn provider_nameservers(provider: &str) -> Option<&'static [&'static str]> {
		match provider {
			"cloudflare" => Some(&["1.1.1.1", "1.0.0.1"]),
			"google" => Some(&["8.8.8.8", "8.8.4.4"]),
			"quad9" => Some(&["9.9.9.9", "149.112.112.112"]),
			"opendns" => Some(&["208.67.222.222", "208.67.220.220"]),
			"adguard" => Some(&["94.140.14.14", "94.140.15.15"]),
			_ => None,
		}
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

pub struct CpuBooster;

impl CpuBooster {
	pub fn set_profile(profile: &str) -> Result<CommandOutput> {
		let governor = Self::map_profile(profile).ok_or_else(|| ModulariteaError::CommandError {
			command: format!("cpu-booster {}", profile),
			exit_code: None,
			stderr: format!("Unsupported CPU profile: {}", profile),
		})?;

		let cmd = format!("pkexec cpupower frequency-set -g {}", governor);

		let output = Command::new("sh")
			.args(["-c", &cmd])
			.output()
			.map_err(|e| ModulariteaError::CommandError {
				command: cmd.clone(),
				exit_code: None,
				stderr: e.to_string(),
			})?;

		let stdout = String::from_utf8_lossy(&output.stdout).to_string();
		let stderr = String::from_utf8_lossy(&output.stderr).to_string();

		if !output.status.success() {
			return Err(ModulariteaError::CommandError {
				command: cmd,
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

	pub fn try_set_profile_and_check(profile: &str) -> bool {
		Self::set_profile(profile).is_ok()
	}

	fn map_profile(profile: &str) -> Option<&'static str> {
		match profile.trim().to_lowercase().as_str() {
			"powersave" => Some("powersave"),
			"performance" | "peformance" => Some("performance"),
			"ondemand" => Some("ondemand"),
			_ => None,
		}
	}
}
