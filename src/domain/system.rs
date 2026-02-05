//! System domain model
//!
//! Represents system state and information.

use serde::{Deserialize, Serialize};

/// System information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemInfo {
    pub hostname: String,
    pub distro_name: String,
    pub distro_version: String,
    pub kernel_version: String,
    pub architecture: String,
}

/// Status of a package
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageStatus {
    Installed { version: String },
    NotInstalled,
    Unknown,
}

/// Status of a service
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Active,
    Inactive,
    Failed,
    Unknown,
}

/// Details about a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDetails {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub status: PackageStatus,
}

/// Details about a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDetails {
    pub name: String,
    pub status: ServiceStatus,
    pub enabled: bool,
}
