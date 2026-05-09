use modularitea_libs::infrastructure::grub::{GrubInstruction, GrubInstructionExecutor};
use serde_json::json;
use std::process;

fn print_json_error(message: &str) {
    println!("{}", json!({ "status": "error", "message": message }));
}

fn parse_dimensions(w_s: Option<&String>, h_s: Option<&String>) -> Result<(u32, u32), String> {
    match (w_s, h_s) {
        (None, None) => Ok((1920, 1080)),
        (Some(w_str), Some(h_str)) => {
            let w: u32 = w_str
                .parse()
                .map_err(|_| format!("invalid WIDTH: {}", w_str))?;
            let h: u32 = h_str
                .parse()
                .map_err(|_| format!("invalid HEIGHT: {}", h_str))?;
            if w == 0 || w > 7680 || h == 0 || h > 4320 {
                return Err("resolution dimensions out of range".into());
            }
            Ok((w, h))
        }
        _ => Err("WIDTH and HEIGHT must be passed together, or omitted for default 1920×1080".into()),
    }
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

    let argv: Vec<String> = std::env::args().collect();
    let width_height = match parse_dimensions(argv.get(3), argv.get(4)) {
        Ok(pair) => pair,
        Err(e) => {
            print_json_error(&format!(
                "{e}. Usage: modularitea-grub <themes_dir> <theme_name> [<width> <height>]"
            ));
            process::exit(1);
        }
    };

    let grub =
        GrubInstruction::with_themes_dir(themes_dir).set_screen_resolution(width_height.0, width_height.1);

    match grub.apply_grub_theme(&theme_name) {
        Ok(_) => println!("ok"),
        Err(e) => {
            print_json_error(&e.to_string());
            process::exit(1);
        }
    }
}
