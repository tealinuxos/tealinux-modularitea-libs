use clap::{Parser, Subcommand};
use log::{LevelFilter, info, warn};
use modularitea_libs::infrastructure::Pacman;
use serde::Serialize;
use std::env;

#[derive(Parser)]
#[command(
    name = "modularitea-profile-installer",
    version,
    about = "Profile installer for Modularitea system",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install { profile: String },

    Uninstall { profile: String },

    ListPackage { profile: String },
}

#[derive(Debug)]
pub struct Profile {
    pub profile: &'static str,
    pub package: &'static [&'static str],
}

pub const PROFILES: &[Profile] = &[
    Profile {
        profile: "devops",
        package: &["docker", "dnsutils", "vim"],
    },
    Profile {
        profile: "cybersecurity",
        package: &["aircrack-ng", "hashcat", "whois"],
    },
];

#[derive(Serialize)]
struct ProfilePackages<'a> {
    profile: &'a str,
    packages: &'a [&'a str],
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

// helper: find a profile by name
fn find_profile(name: &str) -> Option<&'static Profile> {
    PROFILES.iter().find(|p| p.profile == name)
}

fn install_profile(name: &str) {
    match find_profile(name) {
        Some(p) => {
            info!("Installing profile: {}", name);
            let pkgs: Vec<String> = p.package.iter().map(|s| s.to_string()).collect();
            let ret = Pacman::install(&pkgs);

            match ret {
                Ok(retval) => {
                    serde_json::to_string(&retval)
                        .map(|json| println!("{}", json))
                        .unwrap_or_else(|err| {
                            warn!("Failed to serialize output for '{}': {}", name, err);
                            eprintln!("Failed to serialize output.");
                        });
                },
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
        None => {
            warn!("Profile '{}' not found", name);
            eprintln!("Profile '{}' not found.", name);
            std::process::exit(1);
        }
    }
}

fn uninstall_profile(name: &str) {
    match find_profile(name) {
        Some(p) => {
            info!("Uninstalling profile: {}", name);
            let pkgs: Vec<String> = p.package.iter().map(|s| s.to_string()).collect();
            let ret = Pacman::remove(&pkgs, false);

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
        None => {
            warn!("Profile '{}' not found", name);
            eprintln!("Profile '{}' not found.", name);
            std::process::exit(1);
        }
    }
}

fn list_packages(name: &str) {
    match find_profile(name) {
        Some(p) => {
            info!("Listing packages for profile: {}", name);

            let output = ProfilePackages {
                profile: p.profile,
                packages: p.package,
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
        None => {
            warn!("Profile '{}' not found", name);
            eprintln!("Profile '{}' not found.", name);
            std::process::exit(1);
        }
    }
}

fn main() {
    init_logger();
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { profile } => {
            install_profile(&profile);
        }
        Commands::Uninstall { profile } => {
            uninstall_profile(&profile);
        }
        Commands::ListPackage { profile } => {
            list_packages(&profile);
        }
    }
}
