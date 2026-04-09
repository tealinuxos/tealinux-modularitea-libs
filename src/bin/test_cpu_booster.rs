use modularitea_libs::infrastructure::tools_utils::CpuBooster;

fn main() {
	let profile = std::env::args()
		.nth(1)
		.unwrap_or_else(|| "ondemand".to_string());

	match CpuBooster::set_profile(&profile) {
		Ok(output) => {
			println!("CPU profile switched to '{}'", profile);
			if !output.stdout.trim().is_empty() {
				println!("{}", output.stdout);
			}
			if !output.stderr.trim().is_empty() {
				eprintln!("{}", output.stderr);
			}
		}
		Err(err) => {
			eprintln!("Failed switching CPU profile '{}': {:?}", profile, err);
			std::process::exit(1);
		}
	}
}
