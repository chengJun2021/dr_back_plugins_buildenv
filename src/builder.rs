use std::env;
use std::error::Error;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

pub(crate) fn execute_build(build_dir: &Path) -> Result<ExitStatus, Box<dyn Error>> {
	// Path has the npm stuffs in it, we have to graft that back into the process after wiping the rest of the env
	// Rest of the env may contain sensitive stuff like tokens and database access credentials
	//
	// Again, swiss cheese model, if we're not gonna run it on a fully isolated VM,
	// and handling everything as solid blocks of data, we're gonna have to make compromises
	let path = env::var("PATH")?;

	// Isolated running enclave but that may cause dependencies problems
	let mut child = Command::new("sudo")
		.args(&["-u", "root"])
		.current_dir(&build_dir)
		.args(&["webpack", "--mode=production"])
		.env_clear()
		// Reapply PATH, otherwise it can't access coreutils/busybox, npm/node
		.env("PATH", path)
		// Debug outputs, remove in prod?
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.spawn()?;

	return Ok(child.wait()?);
}
