//! GRUB infrastructure
//!
//! Handles GRUB configuration.

use crate::config;
use crate::error::{CommandOutput, ModulariteaError, Result};
use duct::cmd;
use ini::Ini;
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Grub;

impl Grub {
    pub const DEFAULT_CONFIG_PATH: &'static str = "/etc/default/grub";
    // pub const DEFAULT_CONFIG_PATH: &'static str = "/home/fadhil_riyanto_guest/BALI64/tealinux-modularitea-libs2/testfiles/grub";

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

    /// Create a timestamped backup of the GRUB default config and return the backup path.
    pub fn backup_config() -> Result<String> {
        let src = Self::DEFAULT_CONFIG_PATH;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ModulariteaError::InternalError(e.to_string()))?;
        let backup_path = format!("{}{}.bak-{}", src, "", now.as_secs());

        fs::copy(src, &backup_path).map_err(|e| ModulariteaError::FilesystemError {
            operation: format!("backup config to {}", backup_path),
            source: e,
        })?;

        Ok(backup_path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeManifest {
    pub name: String,
    pub version: String,
    pub github_url: Option<String>,
    pub preview_image: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub name_concat: Option<String>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Step {
    CopyDir {
        from: String,
        to: String,
    },
    CopyFile {
        from: String,
        to: String,
    },
    SetGrubVar {
        key: String,
        value: String,
    },
    ReplaceInFile {
        file: String,
        search: String,
        replace: String,
    },
    // future maintainer note: do not add RunCommand OPCODE. thats just security hole.
}

pub struct GrubInstruction {
    pub manifest: Vec<ThemeManifest>,
    pub screen_resolution: Option<(u32, u32)>,
    pub tealinux_grub_changer_manifest_dir: Option<String>,
    pub themes_dir: String,
    pub enable_debug_print: bool,
}

// note about this,
//
// anything that involve changing grub (/etc/grub, or /boot.. vice-versa) need a root account.
// as we know that in order to peform that such thing, we need pkexec. you know what I mean..
// we need external binary, which consist "grub changer only". anyway, binary to change grub theme
// is implemented on ./../bin/modularitea-grub. binary name is "modularitea-grub"
//
// anything that didn't require root permission is available to directly call using libary. here the lists
// - GrubInstructionExecutor::get_all_theme_available(), retrurn a array like data
// - GrubInstructionExecutor::details(theme_name), note: get theme_name from `get_all_theme_available`.
//	this stuff return a details of theme, including their preview if available in form of json.
//
//

pub trait GrubInstructionExecutor {
    // this should be private
    fn load_manifests() -> Result<Vec<ThemeManifest>>;

    fn new() -> Self;
    fn get_all_theme_available(&self) -> Vec<ThemeManifest>;
    fn details(&self, theme_name: &str) -> Option<ThemeManifest>;
    fn apply_grub_theme(&self, theme_name: &str) -> Result<CommandOutput>;
    fn do_backup(&self) -> Result<CommandOutput>;
    fn set_grub_var_with_ini(key: &str, value: &str) -> Result<()>;
    fn reset_grub_config() -> Result<()>;
    fn set_screen_resolution(self, width: u32, height: u32) -> Self;


    /* do not use this method, will removed soon... */
    fn override_tealinux_grub_changer_manifest_dir(self, path: String) -> Self;

    /* do not use this method, will removed soon... */
    fn set_tealinux_grub_changer_manifest_dir(self, path: String) -> Self;
}

const DEFAULT_THEMES_DIR: &str =
    "/home/fadhil_riyanto_guest/BALI64/tealinux-modularitea-libs2/data/grub-theme";

impl GrubInstruction {
    fn set_themes_dir(&mut self, path: String) {
        self.themes_dir = path;
    }

    fn get_themes_dir(&self) -> &str {
        &self.themes_dir
    }

    pub fn with_themes_dir(path: String) -> Self {
        let mut this = GrubInstruction {
            manifest: Vec::new(),
            screen_resolution: None,
            tealinux_grub_changer_manifest_dir: None,
            themes_dir: DEFAULT_THEMES_DIR.to_string(),
            enable_debug_print: false,
        };

        this.set_themes_dir(path);
        this.manifest = Self::load_manifests_from_dir(this.get_themes_dir()).unwrap_or_default();
        print!("Loaded manifests: {:#?}", this.manifest);

        this
    }

    fn load_manifests_from_dir(themes_dir: &str) -> Result<Vec<ThemeManifest>> {
        let mut manifests = Vec::new();

        let read_dir = fs::read_dir(themes_dir).map_err(|e| ModulariteaError::FilesystemError {
            operation: "read themes directory".into(),
            source: e,
        })?;

        print!("DEBUG: {:?}", read_dir);

        for entry in read_dir {
            let entry = entry.map_err(|e| ModulariteaError::FilesystemError {
                operation: "read themes directory entry".into(),
                source: e,
            })?;

            let file_type = entry
                .file_type()
                .map_err(|e| ModulariteaError::FilesystemError {
                    operation: "stat theme entry".into(),
                    source: e,
                })?;

            if !file_type.is_dir() {
                continue;
            }

            let theme_path = entry.path();
            let candidate = theme_path.join("manifest.json");
            println!("content: {}", candidate.display());

            if candidate.is_file() {
                let content = fs::read_to_string(&candidate).map_err(|e| {
                    ModulariteaError::FilesystemError {
                        operation: format!("read manifest {}", candidate.display()),
                        source: e,
                    }
                })?;

                let manifest: ThemeManifest = serde_json::from_str(&content).map_err(|e| {
                    ModulariteaError::InternalError(format!(
                        "failed to parse manifest {}: {}",
                        candidate.display(),
                        e
                    ))
                })?;
                manifests.push(manifest);
            }
        }

        Ok(manifests)
    }
}

impl GrubInstructionExecutor for GrubInstruction {
    fn new() -> Self {
        Self::with_themes_dir(DEFAULT_THEMES_DIR.to_string())
    }

    fn load_manifests() -> Result<Vec<ThemeManifest>> {
        Self::load_manifests_from_dir(DEFAULT_THEMES_DIR)
    }

    fn get_all_theme_available(&self) -> Vec<ThemeManifest> {
        self.manifest
            .iter()
            .cloned()
            .map(|mut m| {
                m.steps.clear();
                m
            })
            .collect()
    }

    fn details(&self, theme_name: &str) -> Option<ThemeManifest> {
        self.manifest.iter().find(|m| m.name == theme_name).cloned()
    }

    fn override_tealinux_grub_changer_manifest_dir(mut self, path: String) -> Self {
        self.tealinux_grub_changer_manifest_dir = Some(path);
        return self;
    }

    fn apply_grub_theme(&self, theme_name: &str) -> Result<CommandOutput> {
        // arr
        let manifest = self
            .manifest
            .iter()
            .find(|m| m.name == theme_name)
            .cloned()
            .ok_or(ModulariteaError::InternalError(format!(
                "theme not found: {}",
                theme_name
            )))?;

        let themes_dir = self
            .tealinux_grub_changer_manifest_dir
            .as_deref()
            .unwrap_or(self.get_themes_dir());

        let expand = |s: &str| -> String {
            if s.contains("${MANIFEST_DIR}") {
                s.replace("${MANIFEST_DIR}", themes_dir)
            } else {
                s.to_string()
            }
        };

        Self::reset_grub_config()?;

        if let Some((width, height)) = self.screen_resolution {
            Self::set_grub_var_with_ini("GRUB_GFXMODE", &format!("{}x{}", width, height))?;
        }

        let mut cmds: Vec<String> = Vec::new();

        for step in manifest.steps.into_iter() {
            match step {
                Step::CopyDir { from, to } => {
                    let src = expand(&from);
                    let cmd = format!("sudo cp -r -u -v '{}' '{}'", src, to);
                    println!("{}", cmd);
                    cmds.push(cmd);
                }
                Step::CopyFile { from, to } => {
                    let src = expand(&from);
                    let cmd = format!("sudo cp '{}' '{}'", src, to);
                    println!("{}", cmd);
                    cmds.push(cmd);
                }
                Step::SetGrubVar { key, value } => {
                    Self::set_grub_var_with_ini(&key, &value)?;
                    println!(
                        "set {}={} in {} using ini parser",
                        key,
                        value,
                        Grub::DEFAULT_CONFIG_PATH
                    );
                }
                Step::ReplaceInFile {
                    file,
                    search,
                    replace,
                } => {
                    let cmd = format!("sudo sed -i 's|{}|{}|g' {}", search, replace, file);
                    println!("{}", cmd);
                    cmds.push(cmd);
                }
            }
        }

        let regen = "sudo grub-mkconfig -o /boot/grub/grub.cfg".to_string();
        println!("{}", regen);
        cmds.push(regen.clone());

        for command in cmds {
            let command = command
                .strip_prefix("sudo ")
                .unwrap_or(&command)
                .to_string();

            let output = cmd("sudo", ["sh", "-c", command.as_str()])
                .run()
                .map_err(|e| ModulariteaError::GrubError {
                    operation: command.clone(),
                    reason: e.to_string(),
                })?;

            let out = String::from_utf8_lossy(&output.stdout).to_string();
            let err = String::from_utf8_lossy(&output.stderr).to_string();

            if !out.is_empty() {
                println!("{}", out);
            }
            if !err.is_empty() {
                eprintln!("{}", err);
            }

            if !output.status.success() {
                return Err(ModulariteaError::GrubError {
                    operation: command.clone(),
                    reason: format!(
                        "exit code: {} stderr: {}",
                        output.status.code().unwrap_or(-1),
                        err
                    ),
                });
            }
        }

        let stdout = "OK".to_string();

        Ok(CommandOutput {
            exit_code: 0,
            stdout,
            stderr: String::new(),
        })
    }

    fn set_grub_var_with_ini(key: &str, value: &str) -> Result<()> {
        let mut conf = Ini::load_from_file(Grub::DEFAULT_CONFIG_PATH).map_err(|e| {
            ModulariteaError::GrubError {
                operation: format!("ini load {}", Grub::DEFAULT_CONFIG_PATH),
                reason: e.to_string(),
            }
        })?;

        let raw_value = if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            value[1..value.len() - 1].to_string()
        } else {
            value.to_string()
        };

        conf.with_section(None::<String>)
            .set(key.to_string(), raw_value.clone());

        // for " symbol
        for (_, prop) in conf.iter_mut() {
            for (_, v) in prop.iter_mut() {
                if !(v.starts_with('"') && v.ends_with('"')) {
                    let escaped = v.replace('"', "\\\"");
                    *v = format!("\"{}\"", escaped);
                }
            }
        }

        conf.write_to_file(Grub::DEFAULT_CONFIG_PATH)
            .map_err(|e| ModulariteaError::GrubError {
                operation: format!("ini write {}", Grub::DEFAULT_CONFIG_PATH),
                reason: e.to_string(),
            })?;

        Ok(())
    }

    fn do_backup(&self) -> Result<CommandOutput> {
        todo!();
    }

    fn reset_grub_config() -> Result<()> {
        Self::set_grub_var_with_ini("GRUB_DEFAULT", "0")?;
        Self::set_grub_var_with_ini("GRUB_TIMEOUT", "5")?;
        Self::set_grub_var_with_ini("GRUB_DISTRIBUTOR", "TealinuxOS")?;
        Self::set_grub_var_with_ini("GRUB_CMDLINE_LINUX_DEFAULT", "loglevel=3 quiet")?;
        Self::set_grub_var_with_ini("GRUB_CMDLINE_LINUX", "rootfstype=ext4")?;
        Self::set_grub_var_with_ini("GRUB_PRELOAD_MODULES", "part_gpt part_msdos")?;
        Self::set_grub_var_with_ini("GRUB_TIMEOUT_STYLE", "menu")?;
        Self::set_grub_var_with_ini("GRUB_TERMINAL_INPUT", "console")?;
        Self::set_grub_var_with_ini("GRUB_GFXMODE", "auto")?;
        Self::set_grub_var_with_ini("GRUB_GFXPAYLOAD_LINUX", "keep")?;
        Self::set_grub_var_with_ini("GRUB_DISABLE_RECOVERY", "true")?;

        Ok(())
    }

    fn set_screen_resolution(mut self, width: u32, height: u32) -> Self {
        self.screen_resolution = Some((width, height));
        self
    }

    fn set_tealinux_grub_changer_manifest_dir(mut self, path: String) -> Self {
        self.tealinux_grub_changer_manifest_dir = Some(path.clone());
        self.set_themes_dir(path);
        self.manifest = Self::load_manifests_from_dir(self.get_themes_dir()).unwrap_or_default();
        self
    }
}
