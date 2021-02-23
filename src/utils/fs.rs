use std::path::Path;
use std::{fs, io};

/// Recursively copy a source directory into the target.
pub(crate) fn rcopy<P: AsRef<Path>>(source_dir: P, target_dir: &Path) -> io::Result<()> {
    for dir in fs::read_dir(source_dir)?
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap())
    {
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

            rcopy(&dir.path(), &current_target)?;
        } else {
            fs::copy(dir.path(), current_target)?;
        }
    }

    return Ok(());
}
