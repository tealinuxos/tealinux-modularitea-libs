//! Profile domain model
//!
//! Represents a TeaLinux configuration profile loaded from TOML.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete profile configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    /// Profile metadata
    pub meta: ProfileMeta,

    /// Packages to install/remove
    #[serde(default)]
    pub packages: PackageConfig,

    /// Services to enable/disable
    #[serde(default)]
    pub services: ServiceConfig,

    /// GRUB configuration
    #[serde(default)]
    pub grub: Option<GrubConfig>,

    /// Filesystem operations
    #[serde(default)]
    pub filesystem: Option<FilesystemConfig>,

    /// Custom settings
    #[serde(default)]
    pub settings: HashMap<String, SettingValue>,
}

/// Profile metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileMeta {
    /// Profile name
    pub name: String,

    /// Profile description
    #[serde(default)]
    pub description: String,

    /// Profile version
    #[serde(default = "default_version")]
    pub version: String,

    /// Author information
    #[serde(default)]
    pub author: String,

    /// Profile category (e.g., "gaming", "development", "minimal")
    #[serde(default)]
    pub category: String,

    /// Dependencies on other profiles
    #[serde(default)]
    pub depends: Vec<String>,

    /// Profiles that conflict with this one
    #[serde(default)]
    pub conflicts: Vec<String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Package configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageConfig {
    /// Packages to install from official repos
    #[serde(default)]
    pub install: Vec<String>,

    /// Packages to install from AUR
    #[serde(default)]
    pub aur: Vec<String>,

    /// Packages to remove
    #[serde(default)]
    pub remove: Vec<String>,

    /// Package groups to install
    #[serde(default)]
    pub groups: Vec<String>,
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceConfig {
    /// Services to enable and start
    #[serde(default)]
    pub enable: Vec<String>,

    /// Services to disable and stop
    #[serde(default)]
    pub disable: Vec<String>,

    /// Services to mask
    #[serde(default)]
    pub mask: Vec<String>,

    /// User services to enable
    #[serde(default)]
    pub user_enable: Vec<String>,

    /// User services to disable
    #[serde(default)]
    pub user_disable: Vec<String>,
}

/// GRUB configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GrubConfig {
    /// Theme name or path
    #[serde(default)]
    pub theme: Option<String>,

    /// Timeout in seconds
    #[serde(default)]
    pub timeout: Option<u32>,

    /// Default boot entry
    #[serde(default)]
    pub default_entry: Option<String>,

    /// Custom GRUB parameters
    #[serde(default)]
    pub cmdline_linux: Vec<String>,

    /// Additional GRUB config entries
    #[serde(default)]
    pub extra_config: HashMap<String, String>,
}

/// Filesystem configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilesystemConfig {
    /// Files to copy
    #[serde(default)]
    pub copy: Vec<FileCopy>,

    /// Symlinks to create
    #[serde(default)]
    pub symlink: Vec<SymlinkEntry>,

    /// Directories to create
    #[serde(default)]
    pub mkdir: Vec<String>,

    /// Files/directories to remove
    #[serde(default)]
    pub remove: Vec<String>,
}

/// File copy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCopy {
    pub src: String,
    pub dest: String,
    #[serde(default)]
    pub mode: Option<u32>,
    #[serde(default)]
    pub owner: Option<String>,
}

/// Symlink entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkEntry {
    pub target: String,
    pub link: String,
}

/// Setting value (can be various types)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SettingValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<SettingValue>),
    Table(HashMap<String, SettingValue>),
}

impl Profile {
    /// Create a new empty profile
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            meta: ProfileMeta {
                name: name.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Check if profile has any package operations
    pub fn has_package_operations(&self) -> bool {
        !self.packages.install.is_empty()
            || !self.packages.aur.is_empty()
            || !self.packages.remove.is_empty()
            || !self.packages.groups.is_empty()
    }

    /// Check if profile has any service operations
    pub fn has_service_operations(&self) -> bool {
        !self.services.enable.is_empty()
            || !self.services.disable.is_empty()
            || !self.services.mask.is_empty()
            || !self.services.user_enable.is_empty()
            || !self.services.user_disable.is_empty()
    }

    /// Check if profile requires root privileges
    pub fn requires_root(&self) -> bool {
        self.has_package_operations()
            || self.has_service_operations()
            || self.grub.is_some()
            || self.filesystem.as_ref().is_some_and(|fs| {
                !fs.copy.is_empty() || !fs.remove.is_empty() || !fs.mkdir.is_empty()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let profile = Profile::new("test-profile");
        assert_eq!(profile.meta.name, "test-profile");
    }

    #[test]
    fn test_empty_profile_no_root() {
        let profile = Profile::new("empty");
        assert!(!profile.requires_root());
    }
}
