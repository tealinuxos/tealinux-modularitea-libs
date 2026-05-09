use modularitea_libs::infrastructure::tools_utils::DnsSwitcher;

fn main() {
	let provider = std::env::args()
		.nth(1)
		.unwrap_or_else(|| "cloudflare".to_string());

	match DnsSwitcher::switch(&provider) {
		Ok(output) => {
			println!("DNS switch success: {}", provider);
			if !output.stdout.trim().is_empty() {
				println!("{}", output.stdout);
			}
			if !output.stderr.trim().is_empty() {
				eprintln!("{}", output.stderr);
			}
		}
		Err(err) => {
			eprintln!("DNS switch failed for '{}': {:?}", provider, err);
			std::process::exit(1);
		}
	}
}
