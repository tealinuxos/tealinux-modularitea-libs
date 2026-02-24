use clap::{Parser, Subcommand};
use modularitea_libs::infrastructure::Pacman;
use std::process::exit;

#[derive(Parser)]
#[command(name = "modularitea-pacman")]
#[command(about = "Modularitea Package Manager Helper (Root Only)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install packages
    Install {
        packages: Vec<String>,
        #[arg(long)]
        aur: bool, // Not used here directly as this is system pacman, but handled by caller? or maybe we wrap yay/paru later? Prompt said "pacman runner".
    },
    /// Remove packages
    Remove {
        packages: Vec<String>,
        #[arg(short, long)]
        recursive: bool,
        #[arg(short, long)]
        force: bool,
    },
    /// Install package groups
    InstallGroup { groups: Vec<String> },
    /// Update database
    UpdateDb,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Install { packages, aur: _ } => {
            // If AUR is requested, this binary might need to handle it differently
            // OR the executor uses a different strategy.
            // For now, mapping to infrastructure::Pacman::install (official repos)
            Pacman::install(&packages)
        }
        Commands::Remove {
            packages,
            recursive,
            force,
        } => Pacman::remove(&packages, recursive, force),
        Commands::InstallGroup { groups } => {
            // Pacman installs groups same as packages usually
            Pacman::install(&groups)
        }
        Commands::UpdateDb => Pacman::update_db(),
    };

    match result {
        Ok(output) => {
            if !output.success() {
                eprintln!("{}", output.stderr);
                exit(output.exit_code);
            }
            println!("{}", output.stdout);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}
