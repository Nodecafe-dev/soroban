#![cfg(test)]

use super::*;
use rand::{thread_rng, RngCore};
use soroban_sdk::testutils::{Accounts};
use soroban_sdk::{AccountId, Env, IntoVal, symbol};
use token::{Client as TokenClient, TokenMetadata};

fn generate_contract_id() -> [u8; 32] {
    let mut id: [u8; 32] = Default::default();
    thread_rng().fill_bytes(&mut id);
    id
}

fn create_token_contract(env: &Env, admin: &AccountId) -> (BytesN<32>, TokenClient) {
    let id = env.register_contract_token(None);
    let token = TokenClient::new(env, &id);
    // decimals, name, symbol don't matter in tests
    token.init(
        &Identifier::Account(admin.clone()),
        &TokenMetadata {
            name: "name".into_val(env),
            symbol: "symbol".into_val(env),
            decimals: 7,
        },
    );
    (id, token)
}

fn create_bet_contract(env: &Env) -> BetContractClient {
    let contract_id = BytesN::from_array(env, &generate_contract_id());
    env.register_contract(&contract_id, BetContract {});

    BetContractClient::new(env, contract_id)
}

struct BetContractTest {
    env: Env,
    bet_owner: AccountId,
    bettors: [AccountId; 3],
    token: TokenClient,
    token_id: BytesN<32>,
    contract: BetContractClient,
    contract_id: Identifier,
}

impl BetContractTest {
    fn setup() -> Self {
        let env: Env = Default::default();

        let bet_owner = env.accounts().generate();

        let bettors = [
            env.accounts().generate(),
            env.accounts().generate(),
            env.accounts().generate(),
        ];

        let token_admin = env.accounts().generate();

        let (token_id, token) = create_token_contract(&env, &token_admin);

        for bettor in bettors.clone() {
            let bettor_id = Identifier::Account(bettor.clone());

            token.with_source_account(&token_admin).mint(
                &Signature::Invoker,
                &BigInt::zero(&env),
                &bettor_id,
                &BigInt::from_u32(&env, 100),
           );
        }

        let contract = create_bet_contract(&env);
        let contract_id = Identifier::Contract(contract.contract_id.clone());
        BetContractTest {
            env,
            bet_owner,
            bettors,
            token,
            token_id,
            contract,
            contract_id: contract_id,
        }
    }

    fn account_id_to_identifier(&self, account_id: &AccountId) -> Identifier {
        Identifier::Account(account_id.clone())
    }

    fn approve_bettor_amount(&self, bettor: &AccountId, amount: &BigInt) {
        self.token.with_source_account(&bettor).approve(
            &Signature::Invoker,
            &BigInt::zero(&self.env),
            &Identifier::Contract(self.contract.contract_id.clone()),
            &amount,
        );
    }

    fn bet_result(&self, bet_id: &u32, bet_result: &BetResult) {
        self.contract
            .with_source_account(&self.bet_owner)
            .bet_result(&bet_id, &bet_result);
    }

    fn create_bet(&self, team_a: &Symbol, team_b: &Symbol) -> u32 {
        self.contract
            .with_source_account(&self.bet_owner)
            .create_bet(&self.token_id, &team_a, &team_b)
    }

    fn place_bet(&self, bettor: &AccountId, bet_id: &u32,  bet_result: &BetResult, amount: &BigInt) {
        self.approve_bettor_amount(&bettor, &amount);

        self.contract
            .with_source_account(&bettor)
            .place_bet(&self.token_id, &bet_id, &bet_result , &amount);
    }
}

#[test]
fn test() {
    let test = BetContractTest::setup();
    let expected_bet_id = 1u32;
    let bet_id = test.create_bet(&symbol!("team_a"), &symbol!("team_b")); 

    assert_eq!(
            bet_id,
            expected_bet_id
        );

   assert_eq!(
        test.contract.get_status(&bet_id), BetStatus::Open
    );

    // place bets
    test.place_bet(&test.bettors[0], &bet_id,  &BetResult::TeamA, &BigInt::from_u32(&test.env, 100));
    test.place_bet(&test.bettors[1], &bet_id,  &BetResult::Draw, &BigInt::from_u32(&test.env, 100));
    test.place_bet(&test.bettors[2], &bet_id,  &BetResult::TeamB, &BigInt::from_u32(&test.env, 100));

    assert_eq!(
        test.token
            .balance(&test.account_id_to_identifier(&test.bettors[0])),
            BigInt::from_u32(&test.env, 0)
    );

    assert_eq!(
        test.token
            .balance(&test.account_id_to_identifier(&test.bettors[1])),
            BigInt::from_u32(&test.env, 0)
    );

    assert_eq!(
        test.token
            .balance(&test.account_id_to_identifier(&test.bettors[2])),
            BigInt::from_u32(&test.env, 0)
    );

    assert_eq!(
        test.token.balance(&test.contract_id),
        BigInt::from_u32(&test.env, 300)
    );

    // close bet
    test.contract.set_status(&bet_id, &BetStatus::Close);
    assert_eq!(
        test.contract.get_status(&bet_id), BetStatus::Close
    );

    // set bet result
    test.bet_result(&bet_id, &BetResult::TeamA);

    // bet gain = PlayerBetAmount + ( (PlayerBetAmount / TotalWinningBetAmount) * TotalLosingBetAmount) = 300
    assert_eq!(
        test.token
            .balance(&test.account_id_to_identifier(&test.bettors[0])),
            BigInt::from_u32(&test.env, 300)
    );

    // other bettors lost
    assert_eq!(
        test.token
            .balance(&test.account_id_to_identifier(&test.bettors[1])),
            BigInt::from_u32(&test.env, 0)
    );

    assert_eq!(
        test.token
            .balance(&test.account_id_to_identifier(&test.bettors[2])),
            BigInt::from_u32(&test.env, 0)
    );

    // contract balance should be 0
    assert_eq!(
        test.token.balance(&test.contract_id),
        BigInt::from_u32(&test.env, 0)
    );

    // bet status should now be complete
    assert_eq!(
        test.contract.get_status(&bet_id), BetStatus::Complete
    );
    
}

/* 
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
*/


