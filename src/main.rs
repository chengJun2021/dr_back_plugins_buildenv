use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

use crate::builder::execute_build;
use crate::utils::fs::rcopy;

mod utils;
mod builder;

fn main() -> Result<(), Box<dyn Error>> {
	let source_path = PathBuf::from("/overlay/");
	let target_path = PathBuf::from("/env");

	rcopy(&source_path, &target_path)?;
	execute_build(&target_path).unwrap();

	Command::new("sleep").arg("86400").output()?;

	return Ok(());
}
