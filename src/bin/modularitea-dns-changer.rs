use modularitea_libs::infrastructure::tools_utils::DnsSwitcher;
use serde::Serialize;

#[derive(Serialize)]
struct SuccessResponse<'a> {
	success: bool,
	provider: &'a str,
	result: &'a str,
	stdout: String,
	stderr: String,
}

#[derive(Serialize)]
struct ErrorResponse<'a> {
	success: bool,
	provider: Option<&'a str>,
	error: String,
}

fn main() {
	let provider = match std::env::args().nth(1) {
		Some(value) if !value.trim().is_empty() => value,
		_ => {
			let err = ErrorResponse {
				success: false,
				provider: None,
				error: "missing provider argument (usage: modularitea-dns-changer <provider>)".to_string(),
			};
			println!(
				"{}",
				serde_json::to_string(&err).unwrap_or_else(|_| {
					"{\"success\":false,\"provider\":null,\"error\":\"serialization_failed\"}".to_string()
				})
			);
			std::process::exit(2);
		}
	};

	match DnsSwitcher::switch(&provider) {
		Ok(output) => {
			let ok = SuccessResponse {
				success: true,
				provider: &provider,
				result: "OK",
				stdout: output.stdout,
				stderr: output.stderr,
			};

			println!(
				"{}",
				serde_json::to_string(&ok).unwrap_or_else(|_| {
					"{\"success\":true,\"result\":\"OK\"}".to_string()
				})
			);
		}
		Err(err) => {
			let fail = ErrorResponse {
				success: false,
				provider: Some(&provider),
				error: format!("{:?}", err),
			};

			println!(
				"{}",
				serde_json::to_string(&fail).unwrap_or_else(|_| {
					"{\"success\":false,\"error\":\"serialization_failed\"}".to_string()
				})
			);
			std::process::exit(1);
		}
	}
}
