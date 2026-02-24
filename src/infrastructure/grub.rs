//! GRUB infrastructure
//!
//! Handles GRUB configuration.

use crate::error::{CommandOutput, ModulariteaError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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
}

impl GrubInstructionExecutor for GrubInstruction {
    fn new() -> Self {
        let manifest = Self::load_manifests().unwrap_or_default();

        GrubInstruction { manifest }
    }

    // this must be private
    fn load_manifests() -> Result<Vec<ThemeManifest>> {
        let themes_dir: &str =
            // this is hardcoded for now
            // don;t ask why, just let it be. 
            // --- IGNORE it ---
            "/home/fadhil_riyanto_guest/BALI64/tealinux-modularitea-libs2/data/grub-theme";
        let mut manifests = Vec::new();

        let read_dir = fs::read_dir(themes_dir).map_err(|e| ModulariteaError::FilesystemError {
            operation: "read themes directory".into(),
            source: e,
        })?;

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
                break;
            }
        }

        Ok(manifests)
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

        let expand = |s: &str| -> String {
            if s.contains("${MANIFEST_DIR}") {
                match std::env::var("TEALINUX_GRUB_CHANGER_MANIFEST_DIR") {
                    Ok(val) => s.replace("${MANIFEST_DIR}", &val),
                    Err(_) => s.to_string(),
                }
            } else {
                s.to_string()
            }
        };

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
                    let val_escaped = value.replace('"', "\\\"");
                    let sed_cmd = format!(
                        "sudo sed -i -E 's|^[[:space:]]*#?[[:space:]]*{}=.*|{}={}|' {}",
                        key,
                        key,
                        val_escaped,
                        Grub::DEFAULT_CONFIG_PATH
                    );
                    println!("{}", sed_cmd);
                    cmds.push(sed_cmd);

                    let ensure_cmd = format!(
                        "sudo grep -q '^[[:space:]]*{}=' {} || sudo sh -c 'echo \"{}={}\" >> {}'",
                        key,
                        Grub::DEFAULT_CONFIG_PATH,
                        key,
                        val_escaped,
                        Grub::DEFAULT_CONFIG_PATH
                    );
                    println!("{}", ensure_cmd);
                    cmds.push(ensure_cmd);
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

        let stdout = "OK".to_string();

        Ok(CommandOutput {
            exit_code: 0,
            stdout,
            stderr: String::new(),
        })
    }

    fn do_backup(&self) -> Result<CommandOutput> {
        todo!();
    }
}
