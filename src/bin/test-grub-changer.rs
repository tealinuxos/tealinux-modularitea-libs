
use modularitea_libs::infrastructure::grub::{GrubInstruction, GrubInstructionExecutor, ThemeManifest};
fn main() {
    let ret = GrubInstruction::new();
    let theme = std::env::args().nth(1).expect("Usage: test-grub-changer <theme_name>");
    println!("{:#?}", ret.apply_grub_theme(&theme));
}