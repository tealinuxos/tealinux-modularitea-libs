//! Task domain model
//!
//! Represents executable tasks derived from a Profile.

use serde::{Deserialize, Serialize};

/// Unique identifier for a task
pub type TaskId = String;

/// A single executable task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,

    pub name: String,

    pub description: String,

    pub action: TaskAction,

    pub requires_root: bool,

    pub depends_on: Vec<TaskId>,

    pub rollback_action: Option<TaskAction>,

    /// Priority (lower = higher priority)
    pub priority: u32,

    /// Execution phase
    pub phase: TaskPhase,
}

/// Task execution phases (determines order)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum TaskPhase {
    /// Pre-installation checks and preparation
    Prepare = 0,
    /// Package installation/removal
    #[default]
    Packages = 1,
    /// Service configuration
    Services = 2,
    /// Filesystem operations
    Filesystem = 3,
    /// System configuration (GRUB, etc.)
    Configure = 4,
    /// Post-installation cleanup
    Cleanup = 5,
}

/// Specific action to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaskAction {
    // ─────────────────────────────────────────────────────────────────────────────
    // Package Actions
    // ─────────────────────────────────────────────────────────────────────────────
    PackageInstall {
        packages: Vec<String>,
        #[serde(default)]
        aur: bool,
    },
    PackageRemove {
        packages: Vec<String>,
        #[serde(default)]
        recursive: bool,
    },
    PackageGroupInstall {
        groups: Vec<String>,
    },

    // ─────────────────────────────────────────────────────────────────────────────
    // Service Actions
    // ─────────────────────────────────────────────────────────────────────────────
    ServiceEnable {
        services: Vec<String>,
        #[serde(default)]
        start_now: bool,
        #[serde(default)]
        user: bool,
    },
    ServiceDisable {
        services: Vec<String>,
        #[serde(default)]
        stop_now: bool,
        #[serde(default)]
        user: bool,
    },
    ServiceMask {
        services: Vec<String>,
    },

    // ─────────────────────────────────────────────────────────────────────────────
    // GRUB Actions
    // ─────────────────────────────────────────────────────────────────────────────
    GrubSetTheme {
        theme: String,
    },
    GrubSetTimeout {
        timeout: u32,
    },
    GrubSetDefault {
        entry: String,
    },
    GrubSetCmdline {
        params: Vec<String>,
    },
    GrubRegenerate,

    // ─────────────────────────────────────────────────────────────────────────────
    // Filesystem Actions
    // ─────────────────────────────────────────────────────────────────────────────
    FileCopy {
        src: String,
        dest: String,
        mode: Option<u32>,
        owner: Option<String>,
    },
    FileRemove {
        path: String,
        recursive: bool,
    },
    DirCreate {
        path: String,
        mode: Option<u32>,
    },
    SymlinkCreate {
        target: String,
        link: String,
    },

    // ─────────────────────────────────────────────────────────────────────────────
    // Settings Actions
    // ─────────────────────────────────────────────────────────────────────────────
    SettingApply {
        key: String,
        value: String,
    },

    // ─────────────────────────────────────────────────────────────────────────────
    // Special Actions
    // ─────────────────────────────────────────────────────────────────────────────
    /// Execute a custom shell command
    ShellCommand {
        command: String,
        #[serde(default)]
        requires_root: bool,
    },

    /// No-op for testing
    Noop,
}

impl Task {
    /// Create a new task with defaults
    pub fn new(id: impl Into<String>, name: impl Into<String>, action: TaskAction) -> Self {
        let requires_root = action.requires_root();
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            action,
            requires_root,
            depends_on: Vec::new(),
            rollback_action: None,
            priority: 100,
            phase: TaskPhase::default(),
        }
    }

    /// Set task description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set task phase
    pub fn with_phase(mut self, phase: TaskPhase) -> Self {
        self.phase = phase;
        self
    }

    /// Add a dependency
    pub fn depends_on(mut self, task_id: impl Into<String>) -> Self {
        self.depends_on.push(task_id.into());
        self
    }

    /// Set rollback action
    pub fn with_rollback(mut self, action: TaskAction) -> Self {
        self.rollback_action = Some(action);
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

impl TaskAction {
    /// Check if this action requires root privileges
    pub fn requires_root(&self) -> bool {
        match self {
            // Package operations always require root
            TaskAction::PackageInstall { .. } => true,
            TaskAction::PackageRemove { .. } => true,
            TaskAction::PackageGroupInstall { .. } => true,

            // System service operations require root
            TaskAction::ServiceEnable { user, .. } => !user,
            TaskAction::ServiceDisable { user, .. } => !user,
            TaskAction::ServiceMask { .. } => true,

            // GRUB operations require root
            TaskAction::GrubSetTheme { .. } => true,
            TaskAction::GrubSetTimeout { .. } => true,
            TaskAction::GrubSetDefault { .. } => true,
            TaskAction::GrubSetCmdline { .. } => true,
            TaskAction::GrubRegenerate => true,

            // Filesystem operations typically require root
            TaskAction::FileCopy { .. } => true,
            TaskAction::FileRemove { .. } => true,
            TaskAction::DirCreate { .. } => true,
            TaskAction::SymlinkCreate { .. } => true,

            // Settings depend on the setting
            TaskAction::SettingApply { .. } => false,

            // Shell commands explicitly specify
            TaskAction::ShellCommand { requires_root, .. } => *requires_root,

            TaskAction::Noop => false,
        }
    }

    /// Get a human-readable description of the action
    pub fn description(&self) -> String {
        match self {
            TaskAction::PackageInstall { packages, aur } => {
                let source = if *aur { "AUR" } else { "repos" };
                format!("Install {} packages from {}", packages.len(), source)
            }
            TaskAction::PackageRemove { packages, .. } => {
                format!("Remove {} packages", packages.len())
            }
            TaskAction::PackageGroupInstall { groups } => {
                format!("Install package groups: {}", groups.join(", "))
            }
            TaskAction::ServiceEnable { services, user, .. } => {
                let scope = if *user { "user" } else { "system" };
                format!("Enable {} services ({})", services.len(), scope)
            }
            TaskAction::ServiceDisable { services, .. } => {
                format!("Disable {} services", services.len())
            }
            TaskAction::ServiceMask { services } => {
                format!("Mask {} services", services.len())
            }
            TaskAction::GrubSetTheme { theme } => format!("Set GRUB theme to {}", theme),
            TaskAction::GrubSetTimeout { timeout } => format!("Set GRUB timeout to {}s", timeout),
            TaskAction::GrubSetDefault { entry } => format!("Set GRUB default to {}", entry),
            TaskAction::GrubSetCmdline { params } => {
                format!("Set GRUB cmdline: {}", params.join(" "))
            }
            TaskAction::GrubRegenerate => "Regenerate GRUB configuration".to_string(),
            TaskAction::FileCopy { src, dest, .. } => format!("Copy {} to {}", src, dest),
            TaskAction::FileRemove { path, .. } => format!("Remove {}", path),
            TaskAction::DirCreate { path, .. } => format!("Create directory {}", path),
            TaskAction::SymlinkCreate { target, link } => format!("Symlink {} -> {}", link, target),
            TaskAction::SettingApply { key, value } => format!("Set {} = {}", key, value),
            TaskAction::ShellCommand { command, .. } => format!("Execute: {}", command),
            TaskAction::Noop => "No operation".to_string(),
        }
    }
}

/// A collection of tasks forming an execution plan
#[derive(Debug, Clone, Default)]
pub struct TaskPlan {
    /// Ordered list of tasks to execute
    pub tasks: Vec<Task>,

    /// Profile name this plan was generated from
    pub profile_name: String,

    /// Whether this is a dry-run plan
    pub dry_run: bool,
}

impl TaskPlan {
    pub fn new(profile_name: impl Into<String>) -> Self {
        Self {
            tasks: Vec::new(),
            profile_name: profile_name.into(),
            dry_run: false,
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Get all tasks that require root
    pub fn root_tasks(&self) -> Vec<&Task> {
        self.tasks.iter().filter(|t| t.requires_root).collect()
    }

    /// Get all tasks that don't require root
    pub fn user_tasks(&self) -> Vec<&Task> {
        self.tasks.iter().filter(|t| !t.requires_root).collect()
    }

    /// Check if any tasks require root
    pub fn requires_root(&self) -> bool {
        self.tasks.iter().any(|t| t.requires_root)
    }

    /// Get task count
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Check if plan is empty
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "install-neovim",
            "Install Neovim",
            TaskAction::PackageInstall {
                packages: vec!["neovim".to_string()],
                aur: false,
            },
        );

        assert_eq!(task.id, "install-neovim");
        assert!(task.requires_root);
    }

    #[test]
    fn test_user_service_no_root() {
        let action = TaskAction::ServiceEnable {
            services: vec!["pipewire".to_string()],
            start_now: true,
            user: true,
        };

        assert!(!action.requires_root());
    }
}
