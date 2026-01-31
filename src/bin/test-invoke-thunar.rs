use std::process::Command;

fn main() {
    Command::new("pkexec")
    .arg("/usr/bin/ls")
    .status()
    .expect("failed to run helper");

}