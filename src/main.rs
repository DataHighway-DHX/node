//! Substrate Node CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;

pub use sc_cli::{VersionInfo, error};

fn main() -> Result<(), error::Error> {
	let version = VersionInfo {
		name: "Substrate Node",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "node",
		author: "MXC Foundation gGmbH",
		description: "node",
		support_url: "https://t.me/mxcfoundation",
		copyright_start_year: 2020,
	};

	command::run(version)
}
