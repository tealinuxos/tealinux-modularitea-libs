use modularitea_libs::infrastructure::tools_utils::StatusState;

fn main() {
	println!("is_swap_enable: {}", StatusState::is_swap_enable());
	println!(
		"current_dns_provider: {}",
		StatusState::get_current_dns_provider()
	);
	println!(
		"cpu_governor_state: {}",
		StatusState::cpu_governor_state()
	);
	println!("current_mirror: {}", StatusState::get_current_mirror());
	println!("current_trash_size: {}", StatusState::get_current_trash_size());
}
