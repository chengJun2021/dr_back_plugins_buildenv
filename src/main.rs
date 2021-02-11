use std::{env, io};
use std::error::Error;
use std::fs;
use std::ops::Not;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn Error>> {
    let source_path = PathBuf::from("/overlay/");
    let target_path = PathBuf::from("/env");

    recursive_copy(&source_path, &target_path)?;

    // Path has the npm stuffs in it, we have to graft that back into the process after wiping the rest of the env
    // Rest of the env may contain sensitive stuff like tokens and database access credentials
    //
    // Again, swiss cheese model, if we're not gonna run it on a fully isolated VM,
    // and handling everything as solid blocks of data, we're gonna have to make compromises
    let path = env::var("PATH")
        .unwrap_or_else(|_| "".to_string());

    let child = Command::new("npm")
        .args(&["run", "build"])
        .env_clear()
        .env("PATH", path)
        .stdout(Stdio::inherit())
        .output()?;

    return Ok(());
}

fn recursive_copy(source_dir: &Path, target_dir: &Path) -> io::Result<()> {
    for dir in fs::read_dir(source_dir)?
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap()) {
        let file_type = dir.file_type()?;
        let current_target = target_dir.join(dir.file_name());
        let current_target_exists = current_target.exists();
        let current_target_is_dir = file_type.is_dir();

        if current_target_exists {
            if !current_target_is_dir {
                fs::remove_file(&current_target)?;
            }
        }

        if current_target_is_dir {
            if !current_target_exists {
                fs::create_dir(&current_target)?;
            }

            recursive_copy(&dir.path(), &current_target)?;
        } else {
            fs::copy(dir.path(), current_target)?;
        }
    }

    return Ok(());
}