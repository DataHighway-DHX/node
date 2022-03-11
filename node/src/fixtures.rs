use datahighway_runtime::{
    AccountId,
    Balance,
};
use codec::{
    Decode, Encode,
};
use hex_literal::hex;
use serde::{Deserialize, Serialize};
use serde_json;
use sp_core::{
    crypto::{UncheckedFrom, UncheckedInto, Wraps},
};
use sp_runtime::{AccountId32};
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

pub fn get_allocation(endowed_accounts_with_balances: Vec<(AccountId32, Balance)>)
	-> Result<Vec<(AccountId32, Balance)>, String> {
    let json_data = &include_bytes!("genesis.json")[..];
    let balances_json: Vec<(AccountId32, String)> = serde_json::from_slice(json_data).unwrap();

    let mut combined_balances: Vec<(AccountId32, Balance)> = vec![];

    if endowed_accounts_with_balances.len() != 0 {
        for e in endowed_accounts_with_balances {
            let account_public_key_endowed: String = e.0.to_string();
            let account_balance_endowed: Balance = e.1.to_string().parse::<Balance>().unwrap();
            let account_ss58_address_endowed: AccountId32 = AccountId32::from_str(&account_public_key_endowed).unwrap();
            combined_balances.push((account_ss58_address_endowed.clone(), account_balance_endowed.clone()));
        }
    }

    if balances_json.len() != 0 {
        for e in balances_json {
            let account_public_key_json: String = e.0.to_string();
            let account_balance_json: Balance = e.1.to_string().parse::<Balance>().unwrap();
            let account_ss58_address_json: AccountId32 = AccountId32::from_str(&account_public_key_json).unwrap();
            let index_of_match = combined_balances.clone().iter().position(|x| x.0 == account_ss58_address_json.clone());

            if let Some(idx) = index_of_match.clone() {
                combined_balances[idx] = (combined_balances[idx].0.clone(), account_balance_json.clone());
            } else {
                combined_balances.push((account_ss58_address_json.clone(), account_balance_json.clone()));
            }
        }
    }

    Ok(combined_balances.clone())
}
