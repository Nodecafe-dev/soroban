#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Accounts, Env, symbol};

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BetContract);
    let client = BetContractClient::new(&env, &contract_id);

    let bet_account_id = env.accounts().generate();

    assert_eq!(
            client
                .with_source_account(&bet_account_id)
                .create_bet(&symbol!("team_a"), &symbol!("team_b")), 
            0
            );
}

