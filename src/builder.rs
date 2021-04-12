use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, fs};

use plugins_commons::model::{BuildContext, SubprocessOutputs};

/// System to execute eslint
pub(crate) fn execute_lint(
    build_dir: &Path,
    ctx: &BuildContext,
) -> Result<(i32, SubprocessOutputs), Box<dyn Error>> {
    let mut cmd = [
        "node",
        "node_modules/eslint/bin/eslint.js",
        "--rulesdir",
        "lib/rules",
        // Prevent argument injection with questionable file names
        // Tested already with eslint -- --help returning this error message:
        // ```
        // No files matching the pattern "--help" were found.
        // Please check for typing mistakes in the pattern.
        // ```
        "--",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect::<Vec<_>>();

    cmd.extend(
        ctx.files
            .keys()
            .filter(|k| k.ends_with("js") || k.ends_with("jsx"))
            .map(|k| format!("src/{}", k)),
    );

    return execute_unprivileged_command(build_dir, &cmd);
}

/// System to setup the build environment and execution of webpack
pub(crate) fn execute_build(build_dir: &Path) -> Result<(i32, SubprocessOutputs), Box<dyn Error>> {
    // Create a user-owned output directory
    let out_path = build_dir.join("dist");
    fs::create_dir(&out_path)?;
    Command::new("chown")
        .arg("bob:builder")
        .arg(&out_path)
        .spawn()?
        .wait()?;

    return execute_unprivileged_command(
        build_dir,
        &[
            "node",
            "node_modules/webpack-cli/bin/cli.js",
            "--mode=production",
        ],
    );
}

fn execute_unprivileged_command<S: AsRef<OsStr>>(
    pwd: &Path,
    subcommand: &[S],
) -> Result<(i32, SubprocessOutputs), Box<dyn Error>> {
    // Path has the npm stuffs in it, we have to graft that back into the process after wiping the rest of the env
    // Rest of the env may contain sensitive stuff like tokens and database access credentials
    //
    // Again, swiss cheese model, if we're not gonna run it on a fully isolated VM,
    // and handling everything as solid blocks of data, we're gonna have to make compromises
    let path = env::var("PATH")?;

    // Isolated running enclave but that may cause dependencies problems
    let out = Command::new("sudo")
        // Wipe all envs
        .env_clear()
        // Reapply PATH, otherwise it can't access coreutils/busybox, npm/node
        .env("PATH", path)
        // Pass PATH through, set user to bob
        .args(&["--preserve-env", "-u", "bob"])
        .current_dir(&pwd)
        .args(subcommand)
        // Debug outputs, remove in prod?
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let pwd_string = pwd.to_string_lossy().to_string() + "/";
    return Ok((
        out.status.code().unwrap_or(1),
        SubprocessOutputs {
            stdout: stdout.replace(&pwd_string, ""),
            stderr: stderr.replace(&pwd_string, ""),
        },
    ));
}
