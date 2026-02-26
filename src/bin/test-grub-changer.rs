
use modularitea_libs::infrastructure::grub::{GrubInstruction, GrubInstructionExecutor, ThemeManifest};
fn main() {
    let theme = std::env::args().nth(1).expect("Usage: test-grub-changer <theme_name>");
    let ret = GrubInstruction::new()
        .set_screen_resolution(1920, 1080);
    println!("{:#?}", ret.apply_grub_theme(&theme));
}