#![feature(async_closure)]
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::error::Error;

/// Various miscellaneous utilities
mod utils;
/// Code to invoke webpack
mod builder;
/// Code for invoking `tempdir` and `zip`
mod spawner;
/// Utilities for operating the RPC server.
mod server;

/// Main function, initializes loggers and the socket server.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	env_logger::init();

	server::listen(6969).await
}
