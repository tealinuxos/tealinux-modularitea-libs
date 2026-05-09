use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::os::unix::fs as unix_fs;

#[cfg(unix)]

pub struct OsHelper;

impl OsHelper {
    // recurse but verbose.
    pub fn cp_ruv<S: AsRef<Path>, D: AsRef<Path>>(
        src: S,
        dst: D,
        verbose: bool,
    ) -> io::Result<()> {
        let src = src.as_ref();
        let dst = dst.as_ref();

        if !src.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "source not found"));
        }

        let mut command = duct::cmd!("cp", "-r", "-u");
        if verbose {
            command = command.arg("-v");
        }
        let status = command.arg(src).arg(dst).run()?;

        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("cp exited with status {}", status),
            ));
        }
        Ok(())
    }
}
