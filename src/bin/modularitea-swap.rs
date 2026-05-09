use modularitea_libs::infrastructure::{mode, Swap};
use serde::Serialize;
use std::env;
use std::process::exit;

#[derive(Serialize)]
struct SuccessResponse {
    success: bool,
    action: &'static str,
    exit_code: i32,
    stdout: String,
    stderr: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    action: Option<&'static str>,
    error: String,
}

fn print_json<T: Serialize>(value: &T, is_error: bool) {
    match serde_json::to_string(value) {
        Ok(json) => {
            if is_error {
                eprintln!("{}", json);
            } else {
                println!("{}", json);
            }
        }
        Err(err) => {
            let fallback = format!(
                "{{\"success\":false,\"error\":\"json serialization failed: {}\"}}",
                err
            );
            eprintln!("{}", fallback);
        }
    }
}

fn main() {
    let arg = env::args().nth(1).unwrap_or_else(|| {
        let out = ErrorResponse {
            success: false,
            action: None,
            error: "missing argument (use 'on' or 'off')".to_string(),
        };
        print_json(&out, true);
        exit(2);
    });

    let (mode_val, action_name) = match arg.as_str() {
        "on" | "enable" => (mode::enable, "enable"),
        "off" | "disable" => (mode::disable, "disable"),
        other => {
            let out = ErrorResponse {
                success: false,
                action: None,
                error: format!("invalid argument: {} (use 'on' or 'off')", other),
            };
            print_json(&out, true);
            exit(2);
        }
    };

    match Swap::set(mode_val) {
        Ok(output) => {
            let out = SuccessResponse {
                success: true,
                action: action_name,
                exit_code: output.exit_code,
                stdout: output.stdout,
                stderr: output.stderr,
            };
            print_json(&out, false);
        }
        Err(err) => {
            let out = ErrorResponse {
                success: false,
                action: Some(action_name),
                error: err.to_string(),
            };
            print_json(&out, true);
            exit(1);
        }
    }
}
