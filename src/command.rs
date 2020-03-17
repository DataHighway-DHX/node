use sp_consensus_aura::sr25519::{AuthorityPair as AuraPair};
use crate::{
    chain_spec,
    cli::Cli,
    service,
};
use sc_cli::VersionInfo;

/// Parse and run command line arguments
pub fn run(version: VersionInfo) -> sc_cli::Result<()> {
    let opt = sc_cli::from_args::<Cli>(&version);

    let mut config = sc_service::Configuration::from_version(&version);

    match opt.subcommand {
        Some(subcommand) => {
            subcommand.init(&version)?;
            subcommand.update_config(&mut config, chain_spec::load_spec, &version)?;
            subcommand.run(config, |config: _| Ok(new_full_start!(config).0))
        }
        None => {
            opt.run.init(&version)?;
            opt.run.update_config(&mut config, chain_spec::load_spec, &version)?;
            opt.run.run(config, service::new_light, service::new_full, &version)
        }
    }
}
