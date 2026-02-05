//! Task Planner
//!
//! Generates a TaskPlan from a Profile.

use crate::domain::{Profile, Task, TaskAction, TaskPhase, TaskPlan};
use crate::error::Result;

/// Planner that converts a Profile into a TaskPlan
pub struct TaskPlanner;

impl TaskPlanner {
    /// Create a plan from a profile
    pub fn plan(profile: &Profile) -> Result<TaskPlan> {
        let mut plan = TaskPlan::new(&profile.meta.name);

        // 1. Package Operations
        Self::plan_packages(profile, &mut plan)?;

        // 2. Filesystem Operations
        Self::plan_filesystem(profile, &mut plan)?;

        // 3. Service Operations
        Self::plan_services(profile, &mut plan)?;

        // 4. GRUB Configuration
        Self::plan_grub(profile, &mut plan)?;

        // 5. Custom Settings
        Self::plan_settings(profile, &mut plan)?;

        // Sort tasks by phase and priority
        plan.tasks
            .sort_by(|a, b| a.phase.cmp(&b.phase).then(a.priority.cmp(&b.priority)));

        Ok(plan)
    }

    fn plan_packages(profile: &Profile, plan: &mut TaskPlan) -> Result<()> {
        let pkg_cfg = &profile.packages;

        // Remove packages
        if !pkg_cfg.remove.is_empty() {
            plan.add_task(
                Task::new(
                    "pkg-remove",
                    "Remove packages",
                    TaskAction::PackageRemove {
                        packages: pkg_cfg.remove.clone(),
                        recursive: true,
                    },
                )
                .with_phase(TaskPhase::Packages)
                .with_priority(10),
            );
        }

        // Install packages (official)
        if !pkg_cfg.install.is_empty() {
            plan.add_task(
                Task::new(
                    "pkg-install",
                    "Install official packages",
                    TaskAction::PackageInstall {
                        packages: pkg_cfg.install.clone(),
                        aur: false,
                    },
                )
                .with_phase(TaskPhase::Packages)
                .with_priority(20),
            );
        }

        // Install package groups
        if !pkg_cfg.groups.is_empty() {
            plan.add_task(
                Task::new(
                    "pkg-groups",
                    "Install package groups",
                    TaskAction::PackageGroupInstall {
                        groups: pkg_cfg.groups.clone(),
                    },
                )
                .with_phase(TaskPhase::Packages)
                .with_priority(25),
            );
        }

        // Install AUR packages
        if !pkg_cfg.aur.is_empty() {
            plan.add_task(
                Task::new(
                    "pkg-aur",
                    "Install AUR packages",
                    TaskAction::PackageInstall {
                        packages: pkg_cfg.aur.clone(),
                        aur: true,
                    },
                )
                .with_phase(TaskPhase::Packages)
                // AUR packages might depend on official ones, so lower priority (higher number)
                .with_priority(30),
            );
        }

        Ok(())
    }

    fn plan_services(profile: &Profile, plan: &mut TaskPlan) -> Result<()> {
        let svc_cfg = &profile.services;

        // Mask services
        if !svc_cfg.mask.is_empty() {
            plan.add_task(
                Task::new(
                    "svc-mask",
                    "Mask services",
                    TaskAction::ServiceMask {
                        services: svc_cfg.mask.clone(),
                    },
                )
                .with_phase(TaskPhase::Services)
                .with_priority(10),
            );
        }

        // Disable services
        if !svc_cfg.disable.is_empty() {
            plan.add_task(
                Task::new(
                    "svc-disable",
                    "Disable system services",
                    TaskAction::ServiceDisable {
                        services: svc_cfg.disable.clone(),
                        stop_now: true,
                        user: false,
                    },
                )
                .with_phase(TaskPhase::Services)
                .with_priority(20),
            );
        }

        // Enable services
        if !svc_cfg.enable.is_empty() {
            plan.add_task(
                Task::new(
                    "svc-enable",
                    "Enable system services",
                    TaskAction::ServiceEnable {
                        services: svc_cfg.enable.clone(),
                        start_now: true,
                        user: false,
                    },
                )
                .with_phase(TaskPhase::Services)
                .with_priority(30),
            );
        }

        // Disable user services
        if !svc_cfg.user_disable.is_empty() {
            plan.add_task(
                Task::new(
                    "svc-user-disable",
                    "Disable user services",
                    TaskAction::ServiceDisable {
                        services: svc_cfg.user_disable.clone(),
                        stop_now: true,
                        user: true,
                    },
                )
                .with_phase(TaskPhase::Services)
                .with_priority(40),
            );
        }

        // Enable user services
        if !svc_cfg.user_enable.is_empty() {
            plan.add_task(
                Task::new(
                    "svc-user-enable",
                    "Enable user services",
                    TaskAction::ServiceEnable {
                        services: svc_cfg.user_enable.clone(),
                        start_now: true,
                        user: true,
                    },
                )
                .with_phase(TaskPhase::Services)
                .with_priority(50),
            );
        }

        Ok(())
    }

    fn plan_filesystem(profile: &Profile, plan: &mut TaskPlan) -> Result<()> {
        if let Some(fs) = &profile.filesystem {
            // Remove files
            for (i, path) in fs.remove.iter().enumerate() {
                plan.add_task(
                    Task::new(
                        format!("fs-remove-{}", i),
                        format!("Remove {}", path),
                        TaskAction::FileRemove {
                            path: path.clone(),
                            recursive: true,
                        },
                    )
                    .with_phase(TaskPhase::Filesystem)
                    .with_priority(10),
                );
            }

            // Create directories
            for (i, path) in fs.mkdir.iter().enumerate() {
                plan.add_task(
                    Task::new(
                        format!("fs-mkdir-{}", i),
                        format!("Create directory {}", path),
                        TaskAction::DirCreate {
                            path: path.clone(),
                            mode: None, // Use defaults for now
                        },
                    )
                    .with_phase(TaskPhase::Filesystem)
                    .with_priority(20),
                );
            }

            // Copy files
            for (i, copy) in fs.copy.iter().enumerate() {
                plan.add_task(
                    Task::new(
                        format!("fs-copy-{}", i),
                        format!("Copy {} to {}", copy.src, copy.dest),
                        TaskAction::FileCopy {
                            src: copy.src.clone(),
                            dest: copy.dest.clone(),
                            mode: copy.mode,
                            owner: copy.owner.clone(),
                        },
                    )
                    .with_phase(TaskPhase::Filesystem)
                    .with_priority(30),
                );
            }

            // Create symlinks
            for (i, symlink) in fs.symlink.iter().enumerate() {
                plan.add_task(
                    Task::new(
                        format!("fs-symlink-{}", i),
                        format!("Symlink {} -> {}", symlink.link, symlink.target),
                        TaskAction::SymlinkCreate {
                            target: symlink.target.clone(),
                            link: symlink.link.clone(),
                        },
                    )
                    .with_phase(TaskPhase::Filesystem)
                    .with_priority(40),
                );
            }
        }
        Ok(())
    }

    fn plan_grub(profile: &Profile, plan: &mut TaskPlan) -> Result<()> {
        if let Some(grub) = &profile.grub {
            // Theme
            if let Some(theme) = &grub.theme {
                plan.add_task(
                    Task::new(
                        "grub-theme",
                        "Set GRUB theme",
                        TaskAction::GrubSetTheme {
                            theme: theme.clone(),
                        },
                    )
                    .with_phase(TaskPhase::Configure)
                    .with_priority(10),
                );
            }

            // Timeout
            if let Some(timeout) = grub.timeout {
                plan.add_task(
                    Task::new(
                        "grub-timeout",
                        "Set GRUB timeout",
                        TaskAction::GrubSetTimeout { timeout },
                    )
                    .with_phase(TaskPhase::Configure)
                    .with_priority(20),
                );
            }

            // Default Entry
            if let Some(entry) = &grub.default_entry {
                plan.add_task(
                    Task::new(
                        "grub-default",
                        "Set GRUB default entry",
                        TaskAction::GrubSetDefault {
                            entry: entry.clone(),
                        },
                    )
                    .with_phase(TaskPhase::Configure)
                    .with_priority(30),
                );
            }

            // Cmdline
            if !grub.cmdline_linux.is_empty() {
                plan.add_task(
                    Task::new(
                        "grub-cmdline",
                        "Set GRUB kernel parameters",
                        TaskAction::GrubSetCmdline {
                            params: grub.cmdline_linux.clone(),
                        },
                    )
                    .with_phase(TaskPhase::Configure)
                    .with_priority(40),
                );
            }

            // Regenerate - Always do this check last if any grub actions were added
            // But checking if tasks exist is complex here, so we might just add it if `grub` block exists
            // Or only if we added tasks. For now, let's assume if grub section exists, we regenerate.
            plan.add_task(
                Task::new(
                    "grub-gen",
                    "Regenerate GRUB configuration",
                    TaskAction::GrubRegenerate,
                )
                .with_phase(TaskPhase::Configure)
                .with_priority(99),
            );
        }
        Ok(())
    }

    fn plan_settings(profile: &Profile, plan: &mut TaskPlan) -> Result<()> {
        let mut sorted_keys: Vec<_> = profile.settings.keys().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            let value = &profile.settings[key];
            // Convert simple values to string, complex ones might need specific handling
            // For now, simple to_string representation
            let value_str = match value {
                crate::domain::SettingValue::String(s) => s.clone(),
                crate::domain::SettingValue::Integer(i) => i.to_string(),
                crate::domain::SettingValue::Float(f) => f.to_string(),
                crate::domain::SettingValue::Boolean(b) => b.to_string(),
                _ => continue, // Skip complex types for basic settings
            };

            plan.add_task(
                Task::new(
                    format!("setting-{}", key),
                    format!("Apply setting {}", key),
                    TaskAction::SettingApply {
                        key: key.clone(),
                        value: value_str,
                    },
                )
                .with_phase(TaskPhase::Configure),
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Profile, TaskPhase};

    #[test]
    fn test_empty_profile_plan() {
        let profile = Profile::new("empty");
        let plan = TaskPlanner::plan(&profile).unwrap();
        assert_eq!(plan.tasks.len(), 0);
    }

    #[test]
    fn test_priority_sorting() {
        // Create a profile with multiple types of actions
        let mut profile = Profile::new("sorted");
        profile.packages.install = vec!["vim".to_string()]; // Phase: Packages (1), Priority: 20
        profile.services.enable = vec!["ssh".to_string()]; // Phase: Services (2), Priority: 30

        let plan = TaskPlanner::plan(&profile).unwrap();

        assert_eq!(plan.tasks.len(), 2);
        // Sort behavior checks
        assert_eq!(plan.tasks[0].phase, TaskPhase::Packages);
        assert_eq!(plan.tasks[1].phase, TaskPhase::Services);
    }

    #[test]
    fn test_package_remove_before_install() {
        let mut profile = Profile::new("pkg-order");
        profile.packages.install = vec!["new-pkg".to_string()];
        profile.packages.remove = vec!["old-pkg".to_string()];

        let plan = TaskPlanner::plan(&profile).unwrap();

        // Check that removal comes first
        let remove_task = &plan.tasks[0];
        let install_task = &plan.tasks[1];

        assert!(remove_task.name.contains("Remove"));
        assert!(install_task.name.contains("Install"));

        // Priority check (Remove=10, Install=20)
        assert!(remove_task.priority < install_task.priority);
    }

    #[test]
    fn test_root_only_actions() {
        let mut profile = Profile::new("root-services");
        profile.services.enable = vec!["docker".to_string()];

        let plan = TaskPlanner::plan(&profile).unwrap();
        assert!(plan.requires_root());
    }
}
