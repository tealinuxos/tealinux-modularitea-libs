//! Infrastructure module
//!
//! Handles direct OS interactions (Pacman, Systemd, GRUB, Filesystem).
//! These functions are executed by the root binaries or during dry-runs/checks.

pub mod fs;
pub mod grub;
pub mod pacman;
pub mod systemctl;

pub use fs::Fs;
pub use grub::Grub;
pub use pacman::Pacman;
pub use systemctl::Systemctl;
