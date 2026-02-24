
use modularitea_libs::infrastructure::grub::{GrubInstruction, GrubInstructionExecutor, ThemeManifest};
fn main() {
    let ret = GrubInstruction::new();
    print!("{:#?}", ret.apply_grub_theme("minegrub"));
}