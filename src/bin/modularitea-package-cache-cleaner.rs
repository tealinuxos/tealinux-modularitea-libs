use modularitea_libs::infrastructure::PackageCacheCleaner;
use std::process::exit;

fn main() {
    if PackageCacheCleaner::try_clean_and_check() {
        println!("Package cache cleaned");
    } else {
        eprintln!("Package cache cleaning failed");
        exit(1);
    }
}
