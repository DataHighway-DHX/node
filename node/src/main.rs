//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;
mod rpc;

fn main() -> sc_cli::Result<()> {
    let version = sc_cli::VersionInfo {
        name: "DataHighwayChain",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "datahighway",
        author: "MXC Foundation gGmbH",
        description: "datahighway-chain",
        support_url: "https://github.com/DataHighway-DHX/node/issues/new",
        copyright_start_year: 2020,
	};
	
	command::run(version)
}
