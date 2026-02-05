use clap::{Parser, Subcommand};
use modularitea_libs::infrastructure::Grub;
use std::process::exit;

#[derive(Parser)]
#[command(name = "modularitea-grub")]
#[command(about = "Modularitea GRUB Helper (Root Only)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Set GRUB theme
    Theme { path: String },
    /// Set timeout
    Timeout { seconds: u32 },
    /// Set default entry
    Default { entry: String },
    /// Set kernel parameters
    Cmdline { params: Vec<String> },
    /// Regenerate configuration
    Regenerate,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Theme { path } => Grub::set_theme(&path).map(|_| ok_output()),
        Commands::Timeout { seconds } => Grub::set_timeout(seconds).map(|_| ok_output()),
        Commands::Default { entry } => Grub::set_default(&entry).map(|_| ok_output()),
        Commands::Cmdline { params: _ } => {
            // Logic for appending parameters needs implementation in infrastructure/grub.rs first?
            // For now, let's say it's not fully supported or I implement a placeholder
            // actually I didn't implement set_cmdline in infrastructure/grub.rs yet!
            // I should update infrastructure/grub.rs later if needed, but for now I'll just error or todo
            eprintln!("Cmdline modification not fully implemented yet");
            exit(1);
        }
        Commands::Regenerate => Grub::regenerate(),
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

fn ok_output() -> modularitea_libs::error::CommandOutput {
    modularitea_libs::error::CommandOutput {
        exit_code: 0,
        stdout: "Success".to_string(),
        stderr: "".to_string(),
    }
}
