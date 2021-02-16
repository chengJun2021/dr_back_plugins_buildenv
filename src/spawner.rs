extern crate tempdir;
extern crate zip;

use std::error::Error;
use std::fs;
use std::io::{self, BufReader, Cursor};
use std::path::{Path, PathBuf};

use crate::builder::execute_build;
use crate::utils::error::drop_errors_or_default;
use crate::utils::fs::rcopy;
use crate::utils::packet::BuildContext;

use self::tempdir::TempDir;
use self::zip::write::FileOptions;
use self::zip::ZipWriter;

/// The result of the build.
///
/// Defaults to [`BuildStatus::LowLevelError`]
#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum BuildStatus {
	/// A validator detected an error, human friendly description
	ValidationError(String),
	/// The webpack executable returned a non-success exit code.
	WebpackExit {
		/// Exit code, `0` should mean success and a non-zero exit code should be documented by webpack.
		code: i32,
		/// Captured `stdout` of webpack. Can be displayed to the client.
		stdout: Vec<u8>,
		/// Captured `stderr` of webpack. Can be displayed to the client.
		stderr: Vec<u8>,
	},
	/// A more primitive error, details will be emitted into the logs.
	LowLevelError,
	/// The build has succeeded. The buffer is a zip file of all the artefacts.
	Success(Vec<u8>),
}

impl Default for BuildStatus {
	fn default() -> Self {
		BuildStatus::LowLevelError
	}
}

/// Run the build with all scripts and objects in the supplied [`BuildContext`]
pub(crate) async fn spawn(ctx: BuildContext) -> Result<BuildStatus, Box<dyn Error>> {
	let result = tokio::task::spawn_blocking(async move || {
		drop_errors_or_default::<_, Box<dyn Error>>(async {
			let td = TempDir::new("build-env-")?;
			let working_directory: &Path = td.path();

			// Copy node stuffs from pwd to subprocess working dir
			rcopy(".", working_directory)?;

			// Drop build context into our working directory
			let source_directory: PathBuf = working_directory.join("src");
			if !ctx.extract_into(&source_directory).await? {
				return Ok(BuildStatus::ValidationError("Possible path traversal attack detected.".to_string()));
			}

			// This occurs due to io/process errors,
			// in that case the only appropriate solution is to panic and let k8s
			// restart the pod
			let exit = execute_build(working_directory)?;
			if let BuildStatus::WebpackExit { code, .. } = &exit {
				if *code != 0 {
					return Ok(exit);
				}
			}

			// Run packaging utility
			let out_directory: PathBuf = working_directory.join("dist");
			return Ok(BuildStatus::Success(create_archive(&out_directory)?));
		}.await)
	});

	Ok(result.await?.await)
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

				let mut file = BufReader::new(fs::OpenOptions::new()
					.read(true)
					.open(dir.path())?);

				io::copy(&mut file, &mut archive)?;
			}
		}

		archive.finish()?;
	}

	return Ok(zip_buf);
}