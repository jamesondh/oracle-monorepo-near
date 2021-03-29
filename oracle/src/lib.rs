#![allow(clippy::too_many_arguments)]

use near_sdk::{ AccountId, Balance, env, near_bindgen };
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::collections::{ Vector, LookupMap };
use near_sdk::json_types::{ ValidAccountId, U64, U128 };

mod types;
mod data_request;
mod mock_token;
mod fungible_token_receiver;
mod callback_args;
mod mock_requestor;
mod whitelist;
mod oracle_config;
mod storage_manager;

use callback_args::*;

use types::{ Timestamp };
use data_request::{ DataRequest };

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize )]
pub struct Contract {
    pub whitelist: whitelist::Whitelist,
    pub config: oracle_config::OracleConfig,
    pub data_requests: Vector<DataRequest>,
    pub validity_bond: U128,
    pub stake_token: mock_token::Token,
    pub validity_bond_token: mock_token::Token,
    // Storage map
    pub accounts: LookupMap<AccountId, Balance>
}

impl Default for Contract {
    fn default() -> Self {
        env::panic(b"Contract should be initialized before usage")
    }
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        initial_whitelist: Option<Vec<ValidAccountId>>,
        config: oracle_config::OracleConfig
    ) -> Self {
        Self {
            whitelist: whitelist::Whitelist::new(initial_whitelist),
            config,
            data_requests: Vector::new(b"dr".to_vec()),
            validity_bond: 1.into(),
            accounts: LookupMap::new(b"a".to_vec()),
            
            // Mock
            stake_token: mock_token::Token::default_new(b"st".to_vec()),
            validity_bond_token: mock_token::Token::default_new(b"vbt".to_vec()),
        }
    }

    pub fn dr_claim(&mut self, _id: U64) {
        // calculate the amount of tokens the user
        // let dri: DataRequest = self.data_requests.get(id.into()).expect("No dri with such id");
        // let mut user_amount: u128 = 0;
        // let mut rounds_share_validity = 0;

        // let round_zero: DataRequestRound = dri.rounds.get(0).unwrap();
        // // If the initial round was the right answer, receives full validity bond
        // if round_zero.winning_outcome().unwrap() == final_outcome {
        //     // divide validity bond among round 0 stakers
        //     user_amount += dri.validity_bond * round_zero.user_outcome_stake.
        //         get(&env::predecessor_account_id()).unwrap().
        //         get(&final_outcome).unwrap() / round_zero.outcome_stakes.get(&final_outcome).unwrap();
        // } else {
        //     // loop over all the other round and divide validity bond over round who where right
        //     for n in 1..dri.rounds.len() {
        //         if dri.rounds.get(n).unwrap().winning_outcome().unwrap() == final_outcome {
        //             rounds_share_validity += 1;
        //         }
        //     }
        // }

        // for n in 1..dri.rounds.len() {
        //     let current_round: DataRequestRound = dri.rounds.get(n).unwrap();
        //     if current_round.winning_outcome().unwrap() != final_outcome {
        //         continue;
        //     }

        //     if rounds_share_validity > 0 {
        //         // share validity bond
        //         user_amount += dri.validity_bond * current_round.user_outcome_stake.
        //             get(&env::predecessor_account_id()).unwrap().
        //             get(&final_outcome).unwrap() / current_round.outcome_stakes.get(&final_outcome).unwrap() / rounds_share_validity;
        //     }

        //     let losing_round: DataRequestRound = dri.rounds.get(n).unwrap();
        //     // original winning stake
        //     user_amount += current_round.user_outcome_stake.
        //       get(&env::predecessor_account_id()).unwrap().
        //       get(&final_outcome).unwrap();

        //     // add losing stakes
        //     user_amount += user_amount * losing_round.outcome_stakes.get(
        //         &losing_round.winning_outcome().unwrap()
        //     ).unwrap() / current_round.total;
        // }
    }
}

impl Contract {
    fn assert_gov(&self) {
        assert_eq!(
            self.config.gov, 
            env::predecessor_account_id(), 
            "This method is only callable by the governance contract {}",
            self.config.gov
        );
    }
}