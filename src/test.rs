#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Accounts, Env, symbol};

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BetContract);
    let client = BetContractClient::new(&env, &contract_id);

    let bet_account_id = env.accounts().generate();
    let expected_bet_id = 1u32;

    assert_eq!(
            client
                .with_source_account(&bet_account_id)
                .create_bet(&symbol!("team_a"), &symbol!("team_b")), 
                expected_bet_id
            );

    assert_eq!(
        client.get_status(&expected_bet_id), BetStatus::Open
    );

    client.set_status(&expected_bet_id, &BetStatus::Close);
    assert_eq!(
        client.get_status(&expected_bet_id), BetStatus::Close
    );

}


#[test]
#[should_panic]
fn test_panic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BetContract);
    let client = BetContractClient::new(&env, &contract_id);

    let bet_account_id = env.accounts().generate();
    let unexpected_bet_id = 100u32;

    assert_eq!(
            client
                .with_source_account(&bet_account_id)
                .create_bet(&symbol!("team_a"), &symbol!("team_b")), 
                unexpected_bet_id
            );

    assert_eq!(
        client.get_status(&unexpected_bet_id), BetStatus::Open
    );

}


