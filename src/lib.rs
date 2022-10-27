#![no_std]

use soroban_sdk::{contracterror, contractimpl, contracttype, vec, panic_error, symbol, Address, BigInt, BytesN, Env, Symbol, Vec};

mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}

use token::{Identifier, Signature};

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    BetNotFound = 1,
    InvalidBetStatus = 2,
    BetOwnerNotFound = 3,
    BetActionNotAllowed = 4
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum BetResult {
    TeamA,
    TeamB,
    Draw
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum BetStatus {
    Open,
    Close,
    Complete
}

#[contracttype]
#[derive(Clone)]
pub struct Bettor {
    pub bettor_id: Address,
    pub bet_result: BetResult,
    pub amount: BigInt
}

#[contracttype]
#[derive(Clone)]
pub struct Bet {
    pub token: BytesN<32>,
    pub status: BetStatus,
    pub team_a: Symbol,
    pub team_b: Symbol
}

#[contracttype]
// BetOwner : AccountID of the owner of the bet
// Bets : List of available bets
// Bettors: List of placed bets
pub enum DataKey {
    BetOWner(u32),
    Bet(u32),
    Bettors(u32),
}

const BET_ID: Symbol = symbol!("BET_ID");

fn address_to_id(address: Address) -> Identifier {
    match address {
        Address::Account(a) => Identifier::Account(a),
        Address::Contract(c) => Identifier::Contract(c),
    }
}

fn bet_exists(env: &Env, bet_id: u32) -> bool {
    env.data().has(DataKey::Bet(bet_id))    
}

fn get_bet(env: &Env,bet_id: u32) -> Bet {

    if  !bet_exists(&env, bet_id)  {
        panic_error!(&env, Error::BetNotFound);
    }

    let bet = env.data()
        .get_unchecked(DataKey::Bet(bet_id))
        .unwrap(); 

    bet
}

fn get_bet_id(env: &Env) -> u32 {
    let mut bet_id = 
        env.data()
            .get(BET_ID)
            .unwrap_or_else(|| Ok(0u32))  // If no value set, assume 0.
            .unwrap(); 

    bet_id = bet_id + 1;
    env.data().set(BET_ID, &bet_id);
    bet_id
}

fn get_bet_owner_id(env: &Env, bet_id: u32) -> Address {

    if !env.data().has(DataKey::BetOWner(bet_id))  {
        panic_error!(&env, Error::BetOwnerNotFound);
    }   

    let owner_id: Address = 
        env.data()
            .get_unchecked(DataKey::BetOWner(bet_id))
            .unwrap();
    owner_id
}

fn get_bettors(env: &Env, bet_id: u32) -> Vec<Bettor> {
    let bettors = 
        env.data()
            .get(DataKey::Bettors(bet_id))
            .unwrap_or(Ok(vec![env]))
            .unwrap();
    bettors
}

fn get_contract_id(env: &Env) -> Identifier {
    Identifier::Contract(env.get_current_contract().into())
}

fn  decode_result(env: &Env, bet_id: u32, bet_result: BetResult) ->(Vec<Bettor>, BigInt, BigInt) {
    let bettors = get_bettors(&env, bet_id);
    let mut winners: Vec<Bettor> = vec![&env];
    let mut total_winning_bet_amount = BigInt::zero(env);
    let mut total_losing_bet_amount = BigInt::zero(env);

    for bettor in bettors.iter() {
        let unwrapped_bettor = bettor.unwrap();

        if unwrapped_bettor.bet_result == bet_result {
            let winner = unwrapped_bettor.clone();
            winners.push_back(winner);
            total_winning_bet_amount = total_winning_bet_amount + &unwrapped_bettor.amount;
        } else {
            total_losing_bet_amount = total_losing_bet_amount + &unwrapped_bettor.amount;
        }
    }
    (winners, total_winning_bet_amount, total_losing_bet_amount)
}

fn transfer_from_account_to_contract(
    env: &Env,
    token_id: &BytesN<32>,
    from: &Identifier,
    amount: &BigInt,
) {
    let client = token::Client::new(&env, token_id);
    let bet_contract_id = get_contract_id(env);
   
    client.xfer_from(
        &Signature::Invoker,
        &BigInt::zero(env),
        &from,
        &bet_contract_id,
        &amount,
    );

}

fn transfer_from_contract_to_account(env: &Env, token_id: &BytesN<32>, to: &Identifier, amount: &BigInt,) {
    let client = token::Client::new(&env, token_id);
    
    client.xfer(&Signature::Invoker, &BigInt::zero(&env), to, amount);
}

fn update_winners_balance(env: &Env, token: &BytesN<32>,winners: &Vec<Bettor>, total_winning_bet_amount: &BigInt, total_losing_bet_amount: &BigInt) {
    // for all winners, transfer their gain
    for winner in winners.clone() {
        let unwrapped_winner = winner.unwrap();
    
        // gain = PlayerBetAmount + ( (PlayerBetAmount / TotalWinningBetAmount) * TotalLosingBetAmount)
        let gain = &unwrapped_winner.amount + ((&unwrapped_winner.amount / total_winning_bet_amount) * total_losing_bet_amount);
    
        // xfer from gain from contract to  bettor
        let to_id = address_to_id(unwrapped_winner.bettor_id);
        transfer_from_contract_to_account(&env, &token, &to_id, &gain);
    }
}

pub struct BetContract;

#[contractimpl]
impl BetContract {
    pub fn create_bet(
        env: Env,
        token: BytesN<32>,
        team_a: Symbol,
        team_b: Symbol
    ) -> u32 {
        let bet_id = get_bet_id(&env);

        let owner_id = env.invoker();
        env.data().set(DataKey::BetOWner(bet_id), owner_id);

        let status = BetStatus::Open;
        let bet =  Bet {
            token,
            status,
            team_a,
            team_b        };

        env.data()
            .set(
                DataKey::Bet(bet_id),
                bet    
            );    

        bet_id
    }

    // deposit shares into the vault: mints the vault shares to "from"
    pub fn place_bet(env: Env, token: BytesN<32>,
        bet_id: u32, bet_result: BetResult, amount: BigInt)  {

        // bet shoud be in a status that allozs placing bets
        let bet = get_bet(&env, bet_id);

        if BetStatus::Open != bet.status  {
            panic_error!(&env, Error::InvalidBetStatus);
        }

        // bettor should not be the bet owner
        let invoker_id = env.invoker();
        let bet_owner_id = get_bet_owner_id(&env, bet_id);
        if invoker_id == bet_owner_id {
            panic_error!(&env, Error::BetActionNotAllowed);
        }

        // get bettor details
        let bettor_id = env.invoker();

        let bettor = Bettor {
            bettor_id: bettor_id.clone(),
            bet_result,
            amount: amount.clone()
        };  

        // add it to the list
        let mut bettors = get_bettors(&env, bet_id);
        bettors.push_back(bettor);

        env.data()
            .set(
                DataKey::Bettors(bet_id),
                bettors
            );   
        
        // transfer the bet amount to the contract
        let from_id = address_to_id(env.invoker());
        transfer_from_account_to_contract(&env, &token, &from_id, &amount);
    }


    pub fn bet_result(env: Env, bet_id: u32, bet_result: BetResult) {

        if  !bet_exists(&env, bet_id)  {
            panic_error!(&env, Error::BetNotFound);
        } 

        let bet = get_bet(&env, bet_id);
        if BetStatus::Close != bet.status {
            panic_error!(&env, Error::InvalidBetStatus);
        }

        // invoker must be the bet owner
        let invoker_id = env.invoker();
        let bet_owner_id = get_bet_owner_id(&env, bet_id);
        if invoker_id != bet_owner_id {
            panic_error!(&env, Error::BetActionNotAllowed);
        }
  
        let (winners, total_winning_bet_amount, total_losing_bet_amount) = decode_result(&env, bet_id, bet_result);
 
        update_winners_balance(&env, &bet.token, &winners, &total_winning_bet_amount, &total_losing_bet_amount);

    }

    pub fn get_status(env: Env, bet_id: u32) -> BetStatus {

        let bet = get_bet(&env, bet_id);
        bet.status
    }

    pub fn set_status(env: Env, bet_id: u32, status: BetStatus) {
        
        let mut bet = get_bet(&env, bet_id);
        bet.status = status;

        env.data()
        .set(
            DataKey::Bet(bet_id),
            bet    
        );   
    }

}

mod test;