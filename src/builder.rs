use std::error::Error;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, fs};

use plugins_commons::model::{Base64Encoded, WebpackOutputs};

/// System to setup the build environment for webpack
pub(crate) fn execute_build(build_dir: &Path) -> Result<(i32, WebpackOutputs), Box<dyn Error>> {
    // Path has the npm stuffs in it, we have to graft that back into the process after wiping the rest of the env
    // Rest of the env may contain sensitive stuff like tokens and database access credentials
    //
    // Again, swiss cheese model, if we're not gonna run it on a fully isolated VM,
    // and handling everything as solid blocks of data, we're gonna have to make compromises
    let path = env::var("PATH")?;

    // Create a nobody-owned output directory
    let out_path = build_dir.join("dist");
    fs::create_dir(&out_path)?;
    Command::new("chown")
        .arg("bob:builder")
        .arg(&out_path)
        .spawn()?
        .wait()?;

    // Isolated running enclave but that may cause dependencies problems
    let out = Command::new("sudo")
        // Wipe all envs
        .env_clear()
        // Reapply PATH, otherwise it can't access coreutils/busybox, npm/node
        .env("PATH", path)
        // Pass PATH through, set user to bob
        .args(&["--preserve-env", "-u", "bob"])
        .current_dir(&build_dir)
        .args(&[
            "node",
            "node_modules/webpack-cli/bin/cli.js",
            "--mode=production",
        ])
        // Debug outputs, remove in prod?
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    return Ok((
        out.status.code().unwrap_or(1),
        WebpackOutputs {
            stdout: Base64Encoded::create(&out.stdout),
            stderr: Base64Encoded::create(&out.stderr),
        },
    ));
}
