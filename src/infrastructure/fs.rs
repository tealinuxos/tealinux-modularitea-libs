//! Filesystem infrastructure
//!
//! Handles file operations.

use crate::error::{ModulariteaError, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub struct Fs;

impl Fs {
    pub fn copy(src: &str, dest: &str, mode: Option<u32>) -> Result<()> {
        fs::copy(src, dest).map_err(|e| ModulariteaError::FilesystemError {
            operation: format!("copy {} -> {}", src, dest),
            source: e,
        })?;

        if let Some(m) = mode {
            let metadata = fs::metadata(dest).map_err(|e| ModulariteaError::FilesystemError {
                operation: format!("metadata {}", dest),
                source: e,
            })?;
            let mut perms = metadata.permissions();
            perms.set_mode(m);
            fs::set_permissions(dest, perms).map_err(|e| ModulariteaError::FilesystemError {
                operation: format!("chmod {} -> {:o}", dest, m),
                source: e,
            })?;
        }
        Ok(())
    }

    pub fn mkdir_p(path: &str, mode: Option<u32>) -> Result<()> {
        fs::create_dir_all(path).map_err(|e| ModulariteaError::FilesystemError {
            operation: format!("mkdir -p {}", path),
            source: e,
        })?;

        if let Some(m) = mode {
            let metadata = fs::metadata(path).map_err(|e| ModulariteaError::FilesystemError {
                operation: format!("metadata {}", path),
                source: e,
            })?;
            let mut perms = metadata.permissions();
            perms.set_mode(m);
            fs::set_permissions(path, perms).map_err(|e| ModulariteaError::FilesystemError {
                operation: format!("chmod {} -> {:o}", path, m),
                source: e,
            })?;
        }
        Ok(())
    }

    pub fn remove(path: &str, recursive: bool) -> Result<()> {
        let p = Path::new(path);
        if !p.exists() {
            return Ok(());
        }

        if p.is_dir() && recursive {
            fs::remove_dir_all(path).map_err(|e| ModulariteaError::FilesystemError {
                operation: format!("rm -rf {}", path),
                source: e,
            })
        } else {
            fs::remove_file(path).map_err(|e| ModulariteaError::FilesystemError {
                operation: format!("rm {}", path),
                source: e,
            })
        }
    }

    pub fn symlink(target: &str, link: &str) -> Result<()> {
        // Remove existing link if exists
        if Path::new(link).exists() || fs::symlink_metadata(link).is_ok() {
            fs::remove_file(link).ok(); // ignore error?
        }

        std::os::unix::fs::symlink(target, link).map_err(|e| ModulariteaError::FilesystemError {
            operation: format!("ln -s {} {}", target, link),
            source: e,
        })
    }
}
