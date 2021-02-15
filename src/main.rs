#![feature(async_closure)]
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::error::Error;

mod utils;
mod builder;
mod spawner;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	env_logger::init();

	server::listen(6969).await
}
