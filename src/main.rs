//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;

pub use sc_cli::{VersionInfo, IntoExit, error};

fn main() -> Result<(), cli::error::Error> {
	let version = VersionInfo {
		name: "Substrate Node",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "node",
		author: "MXC Foundation gGmbH",
		description: "node",
		support_url: "https://t.me/mxcfoundation",
	};

	cli::run(std::env::args(), cli::Exit, version)
}
