### soroban-bet-contract
Bet smart contract built with Soroban.

##Bet contract workflow:
This contract is meant to be used for bets on sport events.
At the moment it's limited to bet on the winning team (or a draw).

#1 - Create a bet : fn create_bet
The first step is to create the bet. A bet owner will provide the 2 teams involved in the game.

Once the bet is create, it is assigned a unique bet_id to identify it. 
The status of the bet is open, meaning people can start placing bet.

#2 - Place a bet: fn place_bet
The bet owner is not allowed to place bets. 
Bets can only be placed for bets in status open.
When a user places a bet, he needs to provide the game outcome (winning team or draw) and the amount he is willing to bet.
This amount is transferred to the contract account.

Once the game starts, the bet owner will update the bet status to close. No more bets can be placed.

#3 - Provide bet result: fn bet_result
Once the game is over, the bet owner will provide the result. 
The contract will check all bets that have been placed to identify the winners. 
The gain for the winners is calculated as below:

PlayerBetAmount + ( (PlayerBetAmount / TotalWinningBetAmount) * TotalLosingBetAmount)

##Possible improvements:
- add other types of bet (i.e. game final score, name of the first scorer ...)
- instead of having only the bet owner to provide the bet result define multiple trusted sources. 
Bet result will only be validated if all sources have provided the same result.
These sources could be connected to different sport feeds.
This could be achieved via multisig.


