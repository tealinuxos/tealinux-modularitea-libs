use clap::{Parser, Subcommand};
use log::{info, warn, LevelFilter};
use modularitea_libs::infrastructure::Pacman;
use modularitea_libs::loader::TomlLoader;
use serde::Serialize;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "modularitea-profile-installer",
    version,
    about = "Profile installer for Modularitea system",
    long_about = None
)]
struct Cli {
    /// Directory containing profile TOML files
    #[arg(long, default_value = "/usr/share/tealinux-modularity/profiles")]
    profiles_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install {
        profile: String,
    },

    Uninstall {
        profile: String,
    },

    ListPackage {
        profile: String,
    },

    /// List all available profiles
    List,
}

#[derive(Debug, Serialize)]
struct ProfilePackages {
    profile: String,
    packages: Vec<String>,
}

fn init_logger() {
    let is_dev = env::var("TEALINUX_BUILD")
        .map(|v| v == "dev")
        .unwrap_or(false);

    let mut builder = env_logger::Builder::new();
    builder.filter_level(if is_dev {
        LevelFilter::Trace
    } else {
        LevelFilter::Off
    });

    let _ = builder.try_init();
}

/// Resolve the profile TOML file path.
/// Tries: exact path, then `<profiles_dir>/<name>.toml`.
fn resolve_profile_path(profiles_dir: &Path, name: &str) -> Option<PathBuf> {
    // If name is an explicit path that exists, use it
    let direct = PathBuf::from(name);
    if direct.exists() {
        return Some(direct);
    }

    // Try as filename inside profiles dir
    let toml_path = profiles_dir.join(format!("{}.toml", name));
    if toml_path.exists() {
        return Some(toml_path);
    }

    None
}

fn install_profile(profiles_dir: &Path, name: &str) {
    let path = match resolve_profile_path(profiles_dir, name) {
        Some(p) => p,
        None => {
            warn!("Profile '{}' not found in {:?}", name, profiles_dir);
            eprintln!("Profile '{}' not found.", name);
            std::process::exit(1);
        }
    };

    let profile = match TomlLoader::load(&path) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to load profile '{}': {}", name, e);
            eprintln!("Failed to load profile: {}", e);
            std::process::exit(1);
        }
    };

    info!("Installing profile: {} ({})", profile.meta.name, name);

    let pkgs = &profile.packages.install;
    if pkgs.is_empty() {
        println!("{{\"exit_code\":0,\"stdout\":\"No packages to install\",\"stderr\":\"\"}}");
        return;
    }

    let ret = Pacman::install(pkgs);
    match ret {
        Ok(retval) => {
            serde_json::to_string(&retval)
                .map(|json| println!("{}", json))
                .unwrap_or_else(|err| {
                    warn!("Failed to serialize output for '{}': {}", name, err);
                    eprintln!("Failed to serialize output.");
                });
        }
        Err(reterr) => {
            serde_json::to_string(&reterr)
                .map(|json| println!("{}", json))
                .unwrap_or_else(|err| {
                    warn!("Failed to serialize error for '{}': {}", name, err);
                    eprintln!("Failed to serialize error.");
                });
        }
    }
}

fn uninstall_profile(profiles_dir: &Path, name: &str) {
    let path = match resolve_profile_path(profiles_dir, name) {
        Some(p) => p,
        None => {
            warn!("Profile '{}' not found in {:?}", name, profiles_dir);
            eprintln!("Profile '{}' not found.", name);
            std::process::exit(1);
        }
    };

    let profile = match TomlLoader::load(&path) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to load profile '{}': {}", name, e);
            eprintln!("Failed to load profile: {}", e);
            std::process::exit(1);
        }
    };

    info!("Uninstalling profile: {} ({})", profile.meta.name, name);

    let pkgs = &profile.packages.install;
    if pkgs.is_empty() {
        println!("{{\"exit_code\":0,\"stdout\":\"No packages to uninstall\",\"stderr\":\"\"}}");
        return;
    }

    let ret = Pacman::remove(pkgs, false, false);
    match ret {
        Ok(retval) => {
            serde_json::to_string(&retval)
                .map(|json| println!("{}", json))
                .unwrap_or_else(|err| {
                    warn!("Failed to serialize output for '{}': {}", name, err);
                    eprintln!("Failed to serialize output.");
                });
        }
        Err(reterr) => {
            serde_json::to_string(&reterr)
                .map(|json| println!("{}", json))
                .unwrap_or_else(|err| {
                    warn!("Failed to serialize error for '{}': {}", name, err);
                    eprintln!("Failed to serialize error.");
                });
        }
    }
}

fn list_packages(profiles_dir: &Path, name: &str) {
    let path = match resolve_profile_path(profiles_dir, name) {
        Some(p) => p,
        None => {
            warn!("Profile '{}' not found in {:?}", name, profiles_dir);
            eprintln!("Profile '{}' not found.", name);
            std::process::exit(1);
        }
    };

    let profile = match TomlLoader::load(&path) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to load profile '{}': {}", name, e);
            eprintln!("Failed to load profile: {}", e);
            std::process::exit(1);
        }
    };

    info!("Listing packages for profile: {}", name);

    let output = ProfilePackages {
        profile: profile.meta.name,
        packages: profile.packages.install,
    };

    match serde_json::to_string(&output) {
        Ok(json) => println!("{}", json),
        Err(err) => {
            warn!("Failed to serialize packages for '{}': {}", name, err);
            eprintln!("Failed to serialize package list.");
            std::process::exit(1);
        }
    }
}

fn list_profiles(profiles_dir: &Path) {
    if !profiles_dir.exists() {
        eprintln!("Profiles directory does not exist: {:?}", profiles_dir);
        std::process::exit(1);
    }

    let entries = match std::fs::read_dir(profiles_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to read profiles directory: {}", e);
            std::process::exit(1);
        }
    };

    let mut profiles = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "toml") {
            match TomlLoader::load(&path) {
                Ok(profile) => {
                    let id = path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    profiles.push(serde_json::json!({
                        "id": id,
                        "name": profile.meta.name,
                        "description": profile.meta.description,
                        "package_count": profile.packages.install.len() + profile.packages.aur.len(),
                    }));
                }
                Err(e) => {
                    eprintln!("Warning: failed to parse {:?}: {}", path, e);
                }
            }
        }
    }

    match serde_json::to_string_pretty(&profiles) {
        Ok(json) => println!("{}", json),
        Err(err) => {
            eprintln!("Failed to serialize profiles: {}", err);
            std::process::exit(1);
        }
    }
}

fn main() {
    init_logger();
    let cli = Cli::parse();
    let profiles_dir = &cli.profiles_dir;

    // Also check dev path relative to binary
    let effective_dir = if profiles_dir.exists() {
        profiles_dir.clone()
    } else if let Ok(exe) = std::env::current_exe() {
        // Try ../profiles relative to the binary (dev mode)
        let dev_path = exe
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.join("profiles"));
        dev_path
            .filter(|p| p.exists())
            .unwrap_or(profiles_dir.clone())
    } else {
        profiles_dir.clone()
    };

    match cli.command {
        Commands::Install { profile } => {
            install_profile(&effective_dir, &profile);
        }
        Commands::Uninstall { profile } => {
            uninstall_profile(&effective_dir, &profile);
        }
        Commands::ListPackage { profile } => {
            list_packages(&effective_dir, &profile);
        }
        Commands::List => {
            list_profiles(&effective_dir);
        }
    }
}
