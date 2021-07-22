use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

/// Recursively copy a source directory into the target.
pub(crate) fn rcopy(target_dir: &Path) -> io::Result<()> {
    Command::new("cp")
        .arg("-R")
        .arg("./")
        .arg(target_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    return Ok(());
}
