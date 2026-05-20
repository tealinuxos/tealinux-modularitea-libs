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


pub struct StatusState;

impl StatusState {
	pub fn is_swap_enable() -> bool {
		let content = fs::read_to_string("/proc/swaps").unwrap_or_default();
		content
			.lines()
			.skip(1)
			.any(|line| !line.trim().is_empty())
	}

	pub fn get_current_dns_provider() -> String {
		let content = fs::read_to_string("/etc/resolv.conf").unwrap_or_default();
		let mut nameservers: Vec<String> = Vec::new();

		for line in content.lines() {
			let line = line.trim();
			if line.starts_with("nameserver") {
				if let Some(ip) = line.split_whitespace().nth(1) {
					nameservers.push(ip.to_string());
				}
			}
		}

		let has = |a: &str, b: &str| {
			nameservers.contains(&a.to_string()) && nameservers.contains(&b.to_string())
		};

		if has("8.8.8.8", "8.8.4.4") {
			"google".to_string()
		} else if has("1.1.1.1", "1.0.0.1") {
			"cloudflare".to_string()
		} else if has("9.9.9.9", "149.112.112.112") {
			"quad9".to_string()
		} else if has("208.67.222.222", "208.67.220.220") {
			"opendns".to_string()
		} else if has("94.140.14.14", "94.140.15.15") {
			"adguard".to_string()
		} else {
			"unknown".to_string()
		}
	}

	pub fn cpu_governor_state() -> String {
		let output = duct::cmd!("cpupower", "frequency-info", "-p")
			.stdout_capture()
			.stderr_capture()
			.run();

		if let Ok(output) = output {
			if output.status.success() {
				let stdout = String::from_utf8_lossy(&output.stdout).to_string();
				if let Some(governor) = Self::extract_governor(&stdout) {
					return governor;
				}
			}
		}

		if let Ok(content) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor") {
			let governor = content.trim().to_lowercase();
			if !governor.is_empty() {
				return governor;
			}
		}

		"unknown".to_string()
	}

	// WARN, DO NOT USE THIS METHOD. THIS IS UNRELIABLE.
	pub fn get_current_mirror() -> String {
		let content = fs::read_to_string("/etc/pacman.d/mirrorlist").unwrap_or_default();
		let mut current_country: Option<String> = None;

		for line in content.lines() {
			let trimmed = line.trim();
			if trimmed.starts_with("##") {
				let label = trimmed.trim_start_matches("##").trim();
				if !label.is_empty() {
					let lower = label.to_lowercase();
					let is_header = lower.starts_with("arch linux")
						|| lower.contains("mirrorlist")
						|| lower.contains("generated by reflector");
					if !is_header {
						current_country = Some(label.to_string());
					}
				}
				continue;
			}

			if trimmed.is_empty() || trimmed.starts_with('#') {
				continue;
			}

			if trimmed.to_lowercase().starts_with("server") {
				return current_country
					.unwrap_or_else(|| "unknown".to_string())
					.to_lowercase();
			}
		}

		"unknown".to_string()
	}

	pub fn get_current_trash_size() -> u64 {
		let home = std::env::var("HOME").unwrap_or_default();
		if home.is_empty() {
			return 0;
		}

		let trash_files = Path::new(&home).join(".local/share/Trash/files");
		Self::dir_size(&trash_files)
	}

	fn extract_governor(stdout: &str) -> Option<String> {
		for line in stdout.lines() {
			let lower = line.to_lowercase();
			if !lower.contains("governor") {
				continue;
			}

			if let Some(start) = line.find('"') {
				if let Some(end) = line[start + 1..].find('"') {
					return Some(line[start + 1..start + 1 + end].to_lowercase());
				}
			}

			if let Some(idx) = lower.find("governor") {
				let tail = lower[idx..].split_whitespace().last();
				if let Some(value) = tail {
					return Some(value.to_string());
				}
			}
		}

		None
	}

	fn dir_size(path: &Path) -> u64 {
		let mut total = 0u64;
		let entries = match fs::read_dir(path) {
			Ok(entries) => entries,
			Err(_) => return 0,
		};

		for entry in entries.flatten() {
			let metadata = match entry.metadata() {
				Ok(metadata) => metadata,
				Err(_) => continue,
			};

			if metadata.is_dir() {
				total = total.saturating_add(Self::dir_size(&entry.path()));
			} else {
				total = total.saturating_add(metadata.len());
			}
		}

		total
	}

	
}