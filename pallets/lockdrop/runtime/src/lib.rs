use system::ensure_signed;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256};
use std::time::{SystemTime, UNIX_EPOCH};
use hex::ToHex;

decl_storage! {
	trait Store for Module<T: Config> as LockdropModule {
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		LockdropContract get(something): Option<String>;
	}
}

impl<T: Config> Module<T> {
    async pub fn deploy_contract() {
		let http = web3::transports::Http::new("https://ropsten.infura.io/v3/f201248703994ac2a1f6e6782aedda7a")?;
		let web3 = web3::Web3::new(http);

		let start = SystemTime::now();
		let since_the_epoch = start
			.duration_since(UNIX_EPOCH)
			.expect("Time went backwards");

		let my_account: Address = "0066B0267Bf7003F5Bc20d8b938005d3E0aeae21".parse().unwrap();
		let bytecode = include_str!("./res/contract_token.code");
		let contract = Contract::deploy(web3.eth(), include_bytes!("../src/contract/res/token.json"))?
			.confirmations(0)
			.options(Options::with(|opt| {
				opt.value = Some(5.into());
				opt.gas_price = Some(5.into());
				opt.gas = Some(3_000_000.into());
			}))
			.execute(
				bytecode,
				(since_the_epoch.as_secs();),
				my_account,
			)
			.await?;

		let contract_address_bytes = contract.address().as_bytes();
		let mut s = String::with_capacity(2 * contract_address_bytes.len());
		contract_address_bytes.write_hex(&mut s).expect("Failed to write");
		LockdropContract::put(s);
    }
}
