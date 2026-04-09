use modularitea_libs::infrastructure::tools_utils::MirrorUtils;

fn main() {
	match MirrorUtils::refresh_fastest_mirror() {
		Ok(output) => {
			println!("Mirror refresh success (exit_code={}):", output.exit_code);
			if !output.stdout.trim().is_empty() {
				println!("{}", output.stdout);
			}
			if !output.stderr.trim().is_empty() {
				eprintln!("{}", output.stderr);
			}
		}
		Err(e) => {
			eprintln!("Mirror refresh failed: {:?}", e);
			std::process::exit(1);
		}
	}
}
