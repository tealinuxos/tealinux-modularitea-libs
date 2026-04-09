use modularitea_libs::infrastructure::{mode, Swap};
use std::process::exit;

fn main() {
    match Swap::set(mode::enable) {
        Ok(output) => {
            if output.stdout.trim().is_empty() {
                println!("Swap enabled (zram, persistent)");
            } else {
                println!("{}", output.stdout);
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(1);
        }
    }
}
