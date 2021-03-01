extern crate tempdir;
extern crate zip;

use std::error::Error;
use std::fs;
use std::io::{self, BufReader, Cursor};
use std::path::{Path, PathBuf};

use plugins_commons::model::{Base64Encoded, BuildContext, BuildStatus};

use crate::builder::*;
use crate::utils::fs::rcopy;

use self::tempdir::TempDir;
use self::zip::write::FileOptions;
use self::zip::ZipWriter;

/// Run the build with all scripts and objects in the supplied [`BuildContext`]
pub(crate) fn spawn(mut ctx: BuildContext) -> Result<BuildStatus, Box<dyn Error>> {
    let td = TempDir::new("build-env")?;
    let working_directory = td.path();

    // Copy node stufut fs from pwd to subprocess working dir
    rcopy(Path::new(".").canonicalize().unwrap(), working_directory)?;

    // Drop build context into our working directory
    let source_directory: PathBuf = working_directory.join("src");
    fs::create_dir(&source_directory)?;

    // Sanitize all the keys in a given build context
    {
        let sus_paths = ctx.sanitize();
        if sus_paths.len() > 0 {
            return Ok(BuildStatus::ValidationError(format!(
                "Possible path traversal attack detected.\n{:?}",
                sus_paths
            )));
        };
    }

    ctx.extract_into(&source_directory)?;

    // This occurs due to io/process errors,
    // in that case the only appropriate solution is to panic and let k8s
    // restart the pod
    let (code, eslint_outputs) = execute_lint(working_directory, &ctx)?;
    if code != 0 {
        return Ok(BuildStatus::ESLintExit {
            code,
            eslint_outputs,
        });
    }

    // The error embedded occurs due to io/process errors,
    // in that case the only appropriate solution is to panic and let k8s
    // restart the pod
    let (code, webpack_outputs) = execute_build(working_directory)?;

    if code != 0 {
        return Ok(BuildStatus::WebpackExit {
            code,
            eslint_outputs,
            webpack_outputs,
        });
    }

    // Run packaging utility
    let out_directory: PathBuf = working_directory.join("dist");
    return Ok(BuildStatus::Success {
        zip: Base64Encoded::create(&create_archive(&out_directory)?),
        eslint_outputs,
        webpack_outputs,
    });
}

/// Create a shallow copy of the directory
///
/// Currently only taking the top level files and none of the directories
pub(crate) fn create_archive(out_dir: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut zip_buf = vec![];

    {
        let mut archive = ZipWriter::new(Cursor::new(&mut zip_buf));

        for entry in out_dir.read_dir()? {
            let dir = entry?;
            if dir.metadata()?.is_file() {
                archive.start_file(dir.file_name().to_string_lossy(), FileOptions::default())?;

                let mut file = BufReader::new(fs::OpenOptions::new().read(true).open(dir.path())?);

                io::copy(&mut file, &mut archive)?;
            }
        }

        archive.finish()?;
    }

    return Ok(zip_buf);
}
