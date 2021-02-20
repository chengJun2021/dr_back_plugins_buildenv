#![feature(async_closure)]
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate plugins_commons;
extern crate serde;
extern crate serde_json;

use std::error::Error;

/// Code to invoke webpack
mod builder;
/// Utilities for operating the RPC server.
mod server;
/// Code for invoking `tempdir` and `zip`
mod spawner;
/// Unit tests
#[cfg(test)]
mod test;
/// Various miscellaneous utilities
mod utils;

/// Main function, initializes loggers and the socket server.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    server::listen(6969).await
}
