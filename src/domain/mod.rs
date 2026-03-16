//! Domain models for modularitea-libs
//!
//! Pure data structures without OS logic.

pub mod aur_package;
pub mod profile;
pub mod system;
pub mod task;

pub use aur_package::*;
pub use profile::*;
pub use system::*;
pub use task::*;
