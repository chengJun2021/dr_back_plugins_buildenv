#![feature(async_closure)]

use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

use crate::utils::fs::rcopy;

mod utils;
mod builder;
mod spawner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let source_path = PathBuf::from("/overlay/");
	let target_path = PathBuf::from("/env");

	rcopy(&source_path, &target_path)?;
	spawner::spawn(&target_path).await?;

	Command::new("sleep").arg("86400").output()?;

	return Ok(());
}
