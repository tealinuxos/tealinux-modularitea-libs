use clap::{Parser, Subcommand};
use modularitea_libs::infrastructure::Fs;
use std::process::exit;

#[derive(Parser)]
#[command(name = "modularitea-settings")]
#[command(about = "Modularitea Settings & Filesystem Helper (Root Only)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply a generic setting
    Set {
        key: String,
        value: String,
    },

    // Filesystem subcommands
    FsCopy {
        src: String,
        dest: String,
        #[arg(long)]
        mode: Option<u32>,
        #[arg(long)]
        owner: Option<String>,
    },
    FsMkdir {
        path: String,
        #[arg(long)]
        mode: Option<u32>,
    },
    FsRemove {
        path: String,
        #[arg(long)]
        recursive: bool,
    },
    FsSymlink {
        target: String,
        link: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Set { key: _, value: _ } => {
            // Placeholder: maybe modify a config file?
            // For now just success for "apply"
            Ok(())
        }
        Commands::FsCopy {
            src,
            dest,
            mode,
            owner: _,
        } => {
            // owner setting not implemented in simple Fs::copy yet, ignoring for now or just checking functionality
            Fs::copy(&src, &dest, mode)
        }
        Commands::FsMkdir { path, mode } => Fs::mkdir_p(&path, mode),
        Commands::FsRemove { path, recursive } => Fs::remove(&path, recursive),
        Commands::FsSymlink { target, link } => Fs::symlink(&target, &link),
    };

    match result {
        Ok(_) => {
            println!("Success");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}
