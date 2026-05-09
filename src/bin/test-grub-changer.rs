use modularitea_libs::infrastructure::grub::{GrubInstruction, GrubInstructionExecutor};
fn main() {
    let theme = std::env::args().nth(1);

    let ret = GrubInstruction::with_themes_dir(
        "/home/fadhil_riyanto_guest/BALI64/tealinux-modularitea-libs/data/grub-theme".to_string(),
    )
    .set_screen_resolution(1920, 1080);

    let retq = ret.get_all_theme_available();

    print!("{:?}", retq);
}