//! Task Runner
//!
//! Executes tasks from a TaskPlan.

use crate::domain::{Task, TaskAction, TaskPlan};
use crate::error::Result;
use crate::privilege::PrivilegeRunner;
use tracing::info;

pub struct Executor;

impl Executor {
    /// Execute a task plan
    pub fn execute(plan: &TaskPlan) -> Result<()> {
        info!("Executing plan for profile: {}", plan.profile_name);

        for task in &plan.tasks {
            Self::execute_task(task)?;
        }

        info!("Plan execution completed successfully");
        Ok(())
    }

    /// Execute a single task
    fn execute_task(task: &Task) -> Result<()> {
        info!("Running task: {} ({})", task.name, task.id);

        match &task.action {
            // Package Operations
            TaskAction::PackageInstall { packages, aur } => {
                let action = if *aur { "install-aur" } else { "install" };
                let args: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
                PrivilegeRunner::run_modularitea("pacman", action, &args)?;
            }
            TaskAction::PackageRemove {
                packages,
                recursive,
            } => {
                let mut args = vec!["remove"];
                if *recursive {
                    args.push("--recursive");
                }
                let pkg_strs: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
                args.extend(pkg_strs);
                PrivilegeRunner::run_modularitea("pacman", "remove", &args[1..])?;
            }
            TaskAction::PackageGroupInstall { groups } => {
                let args: Vec<&str> = groups.iter().map(|s| s.as_str()).collect();
                PrivilegeRunner::run_modularitea("pacman", "install-group", &args)?;
            }

            // Service Operations
            TaskAction::ServiceEnable {
                services,
                start_now,
                user,
            } => {
                // If user is true, we might not need root?
                // But prompt says "Privilege Separation... Never run main binary as root".
                // User services can be run without root.
                // For now, I'll dispatch to modularitea-systemctl which should handle user/system distinction
                // or just run systemctl directly if user==true.

                if *user {
                    // Run directly via infrastructure (but infrastructure shouldn't call pkexec?)
                    // Wait, infrastructure modules wrap commands.
                    // If I look at infrastructure/systemctl.rs: it runs `systemctl --user ...`.
                    // This is fine to run from non-root process.
                    crate::infrastructure::Systemctl::enable(services, *start_now, *user)?;
                } else {
                    // System service -> Root
                    let mut args = Vec::new();
                    if *start_now {
                        args.push("--now");
                    }
                    args.push("--"); // Separator
                    let svc_strs: Vec<&str> = services.iter().map(|s| s.as_str()).collect();
                    args.extend(svc_strs);
                    // We need a binary for this. I'll use modularitea-systemctl
                    PrivilegeRunner::run_modularitea("systemctl", "enable", &args)?;
                }
            }
            TaskAction::ServiceDisable {
                services,
                stop_now,
                user,
            } => {
                if *user {
                    crate::infrastructure::Systemctl::disable(services, *stop_now, *user)?;
                } else {
                    let mut args = Vec::new();
                    if *stop_now {
                        args.push("--now");
                    }
                    args.push("--");
                    let svc_strs: Vec<&str> = services.iter().map(|s| s.as_str()).collect();
                    args.extend(svc_strs);
                    PrivilegeRunner::run_modularitea("systemctl", "disable", &args)?;
                }
            }
            TaskAction::ServiceMask { services } => {
                let args: Vec<&str> = services.iter().map(|s| s.as_str()).collect();
                PrivilegeRunner::run_modularitea("systemctl", "mask", &args)?;
            }

            // GRUB Operations
            TaskAction::GrubSetTheme { theme } => {
                PrivilegeRunner::run_modularitea("grub", "theme", &[theme])?;
            }
            TaskAction::GrubSetTimeout { timeout } => {
                PrivilegeRunner::run_modularitea("grub", "timeout", &[&timeout.to_string()])?;
            }
            TaskAction::GrubSetDefault { entry } => {
                PrivilegeRunner::run_modularitea("grub", "default", &[entry])?;
            }
            TaskAction::GrubSetCmdline { params } => {
                let args: Vec<&str> = params.iter().map(|s| s.as_str()).collect();
                PrivilegeRunner::run_modularitea("grub", "cmdline", &args)?;
            }
            TaskAction::GrubRegenerate => {
                PrivilegeRunner::run_modularitea("grub", "regenerate", &[])?;
            }

            // Filesystem Operations
            // These are tricky. Generic FS ops as root?
            // "modularitea-settings" could handle this, or a new "modularitea-fs".
            // For now, let's assume we use generic shell or `modularitea-settings fs ...`
            // Let's create `modularitea-fs` in binaries later? Or put it in settings?
            // The prompt says `infrastructure/fs.rs` exists.
            TaskAction::FileCopy {
                src,
                dest,
                mode,
                owner,
            } => {
                // Pass simple args. JSON might be better but simple args work for basic stuff.
                // modularitea-settings fs copy <src> <dest> [mode] [owner]
                // Or create modularitea-fs
                // I'll assume we made a modularitea-fs binary or similar.
                // Let's use `modularitea-settings` with subnet `fs`.

                // WARNING: Simple arg parsing might be brittle.
                let mut args = vec!["copy", src.as_str(), dest.as_str()];
                let mode_str;
                let owner_str;
                if let Some(m) = mode {
                    mode_str = m.to_string();
                    args.push("--mode");
                    args.push(&mode_str);
                }
                if let Some(o) = owner {
                    owner_str = o.clone();
                    args.push("--owner");
                    args.push(&owner_str);
                }
                PrivilegeRunner::run_modularitea("settings", "fs-copy", &args)?;
            }
            TaskAction::DirCreate { path, mode } => {
                let mut args = vec!["mkdir", path.as_str()];
                let mode_str;
                if let Some(m) = mode {
                    mode_str = m.to_string();
                    args.push("--mode");
                    args.push(&mode_str);
                }
                PrivilegeRunner::run_modularitea("settings", "fs-mkdir", &args)?;
            }
            TaskAction::FileRemove { path, recursive } => {
                let mut args = vec!["remove", path.as_str()];
                if *recursive {
                    args.push("--recursive");
                }
                PrivilegeRunner::run_modularitea("settings", "fs-remove", &args)?;
            }
            TaskAction::SymlinkCreate { target, link } => {
                PrivilegeRunner::run_modularitea(
                    "settings",
                    "fs-symlink",
                    &["link", target.as_str(), link.as_str()],
                )?;
            }

            // Settings
            TaskAction::SettingApply { key, value } => {
                PrivilegeRunner::run_modularitea("settings", "set", &[key, value])?;
            }

            TaskAction::ShellCommand {
                command,
                requires_root,
            } => {
                if *requires_root {
                    PrivilegeRunner::run("sh", &["-c", command])?;
                } else {
                    // Run locally
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg(command)
                        .status()?;
                }
            }

            TaskAction::Noop => {
                info!("No-op task");
            }
        }
        Ok(())
    }
}
