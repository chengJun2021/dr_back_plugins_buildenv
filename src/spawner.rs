extern crate tempdir;

use std::error::Error;
use std::io::{self, BufReader, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, thread};

use plugins_commons::model::{Base64Encoded, BuildContext, BuildStatus};

use crate::builder::*;
use crate::utils::fs::rcopy;

use self::tempdir::TempDir;

/// Run the build with all scripts and objects in the supplied [`BuildContext`]
pub(crate) fn spawn(mut ctx: BuildContext) -> Result<BuildStatus, Box<dyn Error>> {
    let td = TempDir::new("build-env")?;
    let working_directory = td.path();

    // Copy node stufut fs from pwd to subprocess working dir
    rcopy(working_directory)?;

    thread::sleep(Duration::from_secs(60));

    // Drop build context into our working directory
    let source_directory: PathBuf = working_directory.join("src");
    fs::create_dir(&source_directory)?;

    // Sanitize all the keys in a given build context
    {
        let sus_paths = ctx.sanitize();
        if sus_paths.len() > 0 {
            return Ok(BuildStatus::ValidationError {
                message: format!("Possible path traversal attack detected.\n{:?}", sus_paths),
            });
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

    let envs = vec![("APPLICATION_ENTRY_POINT", ctx.find_root_file())];

    // The error embedded occurs due to io/process errors,
    // in that case the only appropriate solution is to panic and let k8s
    // restart the pod
    let (code, webpack_outputs) = execute_build(working_directory, envs)?;

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
        zip: Base64Encoded::create(&create_distribution(&out_directory)?),
        eslint_outputs,
        webpack_outputs,
    });
}

/// Create a HTML bundle of everything that is needed to render the plugin
///
/// Currently only taking the top level files and none of the directories
pub(crate) fn create_distribution(out_dir: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buf = vec![];
    let mut distribution = Cursor::new(&mut buf);

    distribution.write_all(
        b"<!doctype html><html lang='en'>\
        <head>\
            <meta charset='utf-8'>\
            <meta name='viewport' content='width=device-width,initial-scale=1'>\
            <title>A dR Plugin</title>",
    )?;

    for entry in out_dir.read_dir()? {
        let dir = entry?;
        if dir.metadata()?.is_file() {
            let name = dir.file_name();
            let name = name.to_string_lossy();

            let ext = if let Some(ext) = name.split(".").last() {
                ext
            } else {
                continue;
            };
            let mut file = BufReader::new(fs::OpenOptions::new().read(true).open(dir.path())?);

            match ext {
                "css" => {
                    distribution.write_all(b"<style type='text/css'>")?;
                    io::copy(&mut file, &mut distribution)?;
                    distribution.write_all(b"</style>")?;
                }
                "js" => {
                    distribution.write_all(b"<script type='text/javascript'>")?;
                    io::copy(&mut file, &mut distribution)?;
                    distribution.write_all(b"\n</script>")?;
                }
                "txt" => {
                    let mut str = String::new();
                    file.read_to_string(&mut str)?;

                    str = str.replace("<", "&lt;");
                    str = str.replace(">", "&gt;");

                    distribution.write_all(b"<!--\n\n")?;
                    distribution.write_all(str.as_bytes())?;
                    distribution.write_all(b"\n-->")?;
                }
                _ => continue,
            }
        }
    }

    distribution.write_all(b"</head><body></body></html>")?;

    return Ok(buf);
}
