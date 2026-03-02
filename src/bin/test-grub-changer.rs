
use modularitea_libs::infrastructure::grub::{GrubInstruction, GrubInstructionExecutor, ThemeManifest};
fn main() {
    let theme = std::env::args().nth(1).expect("Usage: test-grub-changer <theme_name>");
    let ret = GrubInstruction::new()
        .set_screen_resolution(1920, 1080)
        .override_tealinux_grub_changer_manifest_dir("/home/fadhil_riyanto_guest/BALI64/tealinux-modularitea-libs2/data/grub-theme".to_string());
    println!("{:#?}", ret.apply_grub_theme(&theme));
}