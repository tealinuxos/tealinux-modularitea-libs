use modularitea_libs::infrastructure::grub::{GrubInstruction, GrubInstructionExecutor};
use serde_json::json;
use std::process;

fn print_json_error(message: &str) {
    println!("{}", json!({ "status": "error", "message": message }));
}

fn main() {
    let themes_dir = match std::env::args().nth(1) {
        Some(v) => v,
        None => {
            print_json_error("missing argument: <themes_dir_path>");
            process::exit(1);
        }
    };

    let theme_name = match std::env::args().nth(2) {
        Some(v) => v,
        None => {
            print_json_error("missing argument: <theme_name>");
            process::exit(1);
        }
    };

    let grub = GrubInstruction::with_themes_dir(themes_dir)
        .set_screen_resolution(1920, 1080);

    match grub.apply_grub_theme(&theme_name) {
        Ok(_) => {
            println!("ok");
        }
        Err(e) => {
            print_json_error(&e.to_string());
            process::exit(1);
        }
    }
}
