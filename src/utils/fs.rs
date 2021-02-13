use std::{fs, io};
use std::path::Path;

pub(crate) fn rcopy(source_dir: &Path, target_dir: &Path) -> io::Result<()> {
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

		// When on unix, generate a few symlinks
		#[cfg(any(target_os = "linux", target_os = "macos"))] {
			use std::os::unix::fs as unixfs;

			unixfs::symlink(dir.path(), &current_target)?;
		}

		// Manually copy files and create directories when not on unix
		#[cfg(not(any(target_os = "linux", target_os = "macos")))] {
			if current_target_is_dir {
				if !current_target_exists {
					fs::create_dir(&current_target)?;
				}

				rcopy(&dir.path(), &current_target)?;
			} else {
				fs::copy(dir.path(), current_target)?;
			}
		}
	}

	return Ok(());
}
