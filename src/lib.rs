#![no_std]

use soroban_sdk::{contracterror, contractimpl, contracttype, vec, panic_error, symbol, Address, BigInt, Env, Symbol, Vec};

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    BetNotFound = 1,
    InvalidBetStatus = 2
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
    pub status: BetStatus,
    pub team_a: Symbol,
    pub team_b: Symbol,
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

/*pub trait BetContractTrait {
    fn init_bet(
        e: Env,
        admin: Identifier,
    ) -> Identifier;

    fn place_bet(e: Env, from: Identifier);

    fn bet_result(e: Env, admin: Identifier);

}*/

fn bet_exists(env: &Env, bet_id: u32) -> bool {
    env.data().has(DataKey::Bet(bet_id))    
}

fn get_bet(env: &Env,bet_id: u32) -> Bet {

    let bet = env.data()
        .get_unchecked(DataKey::Bet((bet_id)))
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

fn get_bettors(env: &Env, bet_id: u32) -> Vec<Bettor> {
    let bettors = 
        env.data()
            .get(DataKey::Bettors(bet_id))
            .unwrap_or(Ok(vec![env]))
            .unwrap();
    bettors
}
fn get_num_bettors(env: &Env, bet_id: u32) -> u32 {
    let bettors = get_bettors(&env, bet_id);
    bettors.len()
}

fn  get_winners(env: &Env, bet_id: u32, bet_result: BetResult) -> Vec<Bettor> {
    let bettors = get_bettors(&env, bet_id);
    let mut winners: Vec<Bettor> = vec![&env];
    
    for bettor in bettors.iter() {
        let unwrappwed_bettor = bettor.unwrap();
        if unwrappwed_bettor.bet_result == bet_result {
            // winnder ! 
            let winner = unwrappwed_bettor;
            winners.push_back(winner);
        }
    }
    winners
}

pub struct BetContract;

#[contractimpl]
impl BetContract {
    pub fn create_bet(
        env: Env,
        team_a: Symbol,
        team_b: Symbol
    ) -> u32 {
        let bet_id = get_bet_id(&env);

        let owner_id = env.invoker();
        env.data().set(DataKey::BetOWner(bet_id), owner_id);

        let status = BetStatus::Open;
        let bet =  Bet {
            status,
            team_a,
            team_b,
        };

        env.data()
            .set(
                DataKey::Bet(bet_id),
                bet    
            );    

        bet_id
    }

    pub fn get_status(env: Env, bet_id: u32) -> BetStatus {

        if  !bet_exists(&env, bet_id)  {
            panic_error!(&env, Error::BetNotFound);
        }
        
        let bet = get_bet(&env, bet_id);
        bet.status
    }

    pub fn set_status(env: Env, bet_id: u32, status: BetStatus) {

        if  !bet_exists(&env, bet_id)  {
            panic_error!(&env, Error::BetNotFound);
        }

        let mut bet = get_bet(&env, bet_id);
        bet.status = status;

        env.data()
        .set(
            DataKey::Bet(bet_id),
            bet    
        );   
    }
    // deposit shares into the vault: mints the vault shares to "from"
    pub fn place_bet(env: Env, bet_id: u32, bet_result: BetResult, amount: BigInt)  {
        
        if  bet_exists(&env, bet_id)  {
            panic_error!(&env, Error::BetNotFound);
        } 
        
        let bet = get_bet(&env, bet_id);
        if bet.status != BetStatus::Open {
            panic_error!(&env, Error::InvalidBetStatus);
        }

        let mut bettors = get_bettors(&env, bet_id);

        let bettor_id = env.invoker();
        let bettor = Bettor {
            bettor_id,
            bet_result,
            amount
        };  

        bettors.push_back(bettor);

        env.data()
            .set(
                DataKey::Bettors(bet_id),
                bettors
            );   
        // token xfer
        // Should transfer occurs between the bettor_id and the bet_owner_id
        // or between the bettor_id and the contract_id ?
    }


    pub fn bet_result(env: Env, bet_id: u32, bet_result: BetResult) {
        if  bet_exists(&env, bet_id)  {
            panic_error!(&env, Error::BetNotFound);
        } 
        
        let bet = get_bet(&env, bet_id);
        if bet.status != BetStatus::Close {
            panic_error!(&env, Error::InvalidBetStatus);
        }

        let winners = get_winners(&env, bet_id, bet_result);

        // get num winncers
        let num_winners = winners.len();
        let num_bettors = get_num_bettors(&env, bet_id);

        // for all winners, ransfer their gain

    }
}

mod test;