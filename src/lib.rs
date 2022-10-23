#![no_std]

use soroban_sdk::{contractimpl, contracttype, map, symbol, vec, Address, BigInt, Env, Map, Symbol, Vec};


#[contracttype]
#[derive(Clone)]
pub struct Bettor {
    pub bettor_id: Address,
    pub team_bet: Symbol,
    pub amount: BigInt
}

#[contracttype]
#[derive(Clone)]
pub struct Bet {
    pub team_a: Symbol,
    pub team_b: Symbol,
}

#[contracttype]
// BetOwner : AccountID of the owner of the bet
// Bets : List of available bets
// Bettors: List of placed bets
pub enum DataKey {
    BetOWner(u32),
    Bets,
    Bettors(u32),
}


/*pub trait BetContractTrait {
    fn init_bet(
        e: Env,
        admin: Identifier,
    ) -> Identifier;

    fn place_bet(e: Env, from: Identifier);

    fn bet_result(e: Env, admin: Identifier);

}*/

fn get_bets(
    env: &Env
) -> Map<u32, Bet> {
    env.data()
        .get(DataKey::Bets)
        .unwrap_or(Ok(map![&env])) // if no candidates participated
        .unwrap()
}

fn bet_exists(env: &Env, bet_id: u32) -> bool {
    let bets = get_bets(&env);
    bets.contains_key(bet_id)
}

pub struct BetContract;

#[contractimpl]
impl /*BetContractTrait for */ BetContract {
    pub fn create_bet(
        env: Env,
        team_a: Symbol,
        team_b: Symbol
    ) -> u32 {
        let mut bets = get_bets(&env);
        let bet_id =bets.len();

        let owner_id = env.invoker();
        env.data().set(DataKey::BetOWner(bet_id), owner_id);

        let bet =  Bet {
            team_a,
            team_b,
        };

        bets.set(bet_id, bet);

        env.data()
            .set(
                DataKey::Bets,
                bets    
            );    

        bet_id
    }


    // deposit shares into the vault: mints the vault shares to "from"
    pub fn place_bet(env: Env, bet_id: u32, team_bet: Symbol, amount: BigInt) {
        
        if  !bet_exists(&env, bet_id)  {
            panic!("Bet id does not exist");
        }
    
        let bettor_id = env.invoker();
        let bettor = Bettor {
            bettor_id,
            team_bet,
            amount
        };  

        env.data()
            .set(
                DataKey::Bettors(bet_id),
                bettor
            );    
    }

    // run the lottery
 //   fn bet_result(e: Env, admin: Identifier) {}
}

mod test;