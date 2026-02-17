//! Error types for modularitea-libs
//!
//! Centralized error handling using thiserror for ergonomic error definitions.

use std::path::PathBuf;
use thiserror::Error;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Serialize)]
pub struct CommandErrorReturn {
    pub operation: String,
    pub exit_code: Option<i32>,
    pub stderr: String,
}

impl fmt::Display for CommandErrorReturn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.operation)
    }
}

/// Main error type for the library
#[derive(Error, Debug)]
pub enum ModulariteaError {
    // ─────────────────────────────────────────────────────────────────────────────
    // Loader Errors
    // ─────────────────────────────────────────────────────────────────────────────
    #[error("Failed to read profile file: {path}")]
    ProfileReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse TOML profile: {path}")]
    ProfileParseError {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Profile validation failed: {message}")]
    ProfileValidationError { message: String },

    // ─────────────────────────────────────────────────────────────────────────────
    // Planner Errors
    // ─────────────────────────────────────────────────────────────────────────────
    #[error("Task planning failed: {message}")]
    PlanningError { message: String },

    #[error("Dependency resolution failed: {message}")]
    DependencyError { message: String },

    #[error("Circular dependency detected: {cycle}")]
    CircularDependencyError { cycle: String },

    // ─────────────────────────────────────────────────────────────────────────────
    // Executor Errors
    // ─────────────────────────────────────────────────────────────────────────────
    #[error("Task execution failed: {task_name}")]
    ExecutionError {
        task_name: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Rollback failed for task: {task_name}")]
    RollbackError { task_name: String, reason: String },

    // ─────────────────────────────────────────────────────────────────────────────
    // Infrastructure Errors
    // ─────────────────────────────────────────────────────────────────────────────
    #[error("Pacman operation failed: {0}")]
    PacmanError(CommandErrorReturn),

    #[error("Grub configuration failed: {operation}")]
    GrubError { operation: String, reason: String },

    #[error("Systemctl operation failed: {operation}")]
    SystemctlError {
        operation: String,
        exit_code: Option<i32>,
        stderr: String,
    },

    #[error("Filesystem operation failed: {operation}")]
    FilesystemError {
        operation: String,
        #[source]
        source: std::io::Error,
    },

    // ─────────────────────────────────────────────────────────────────────────────
    // Privilege Errors
    // ─────────────────────────────────────────────────────────────────────────────
    #[error("Privilege escalation failed: {reason}")]
    PrivilegeError { reason: String },

    #[error("pkexec not found in PATH")]
    PkexecNotFound,

    #[error("Polkit authentication was cancelled")]
    PolkitCancelled,

    #[error("Root binary not found: {binary}")]
    RootBinaryNotFound { binary: String },

    // ─────────────────────────────────────────────────────────────────────────────
    // General Errors
    // ─────────────────────────────────────────────────────────────────────────────
    #[error("Command execution failed: {command}")]
    CommandError {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, ModulariteaError>;

/// Execution result with optional output
#[derive(Debug, Clone, Serialize)]
pub struct CommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl CommandOutput {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}
