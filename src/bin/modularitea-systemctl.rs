use clap::{Parser, Subcommand};
use modularitea_libs::infrastructure::Systemctl;
use std::process::exit;

#[derive(Parser)]
#[command(name = "modularitea-systemctl")]
#[command(about = "Modularitea Systemd Helper (Root Only)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Enable {
        services: Vec<String>,
        #[arg(long)]
        now: bool,
    },
    Disable {
        services: Vec<String>,
        #[arg(long)]
        now: bool,
    },
    Mask {
        services: Vec<String>,
    },
    Start {
        services: Vec<String>,
    },
    Stop {
        services: Vec<String>,
    },
    Restart {
        services: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    // Systemctl helper running as root implies user=false for system services.
    // If we wanted to manage user services for a specific user, we'd need more logic (machinectl, sudo -u, etc.)
    // But typically this binary is called by pkexec for SYSTEM services.
    let user = false;

    let result = match cli.command {
        Commands::Enable { services, now } => Systemctl::enable(&services, now, user),
        Commands::Disable { services, now } => Systemctl::disable(&services, now, user),
        Commands::Mask { services } => Systemctl::mask(&services),
        Commands::Start { services } => Systemctl::start(&services, user),
        Commands::Stop { services } => Systemctl::stop(&services, user),
        Commands::Restart { services } => Systemctl::restart(&services, user),
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
