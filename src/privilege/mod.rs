//! Privilege escalation module
//!
//! Handles running commands with root privileges via polkit/pkexec.

pub mod runner;

pub use runner::*;
