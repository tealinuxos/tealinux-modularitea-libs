//! Infrastructure module
//!
//! Handles direct OS interactions (Pacman, Paru, Systemd, GRUB, Filesystem, AUR API).
//! These functions are executed by the root binaries or during dry-runs/checks.

pub mod aur_client;
pub mod fs;
pub mod grub;
pub mod pacman;
pub mod paru;
pub mod systemctl;
pub mod news_parser;
pub mod tools_utils;

pub use aur_client::AurClient;
pub use fs::Fs;
pub use grub::Grub;
pub use pacman::Pacman;
pub use paru::Paru;
pub use systemctl::Systemctl;
pub use tools_utils::{mode, PackageCacheCleaner, Swap, ToolsUtils};
