#![allow(clippy::too_many_arguments)]

use near_sdk::{ AccountId, env, near_bindgen };
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::collections::{ Vector };
use near_sdk::json_types::{ ValidAccountId, U64, U128 };

mod types;
mod data_request;
mod mock_token;
mod fungible_token_receiver;
mod callback_args;
mod mock_requestor;
mod whitelist;
mod oracle_config;

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
    pub token: mock_token::Token,
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
            token: mock_token::Token::default_new(),
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
    
    // TODO: Pass in round as a param for challenges to avoid race conditions
    // TODO: Consume and account for amount
    fn dr_challenge(&mut self, _sender: AccountId, _amount: u128, _payload: ChallengeDataRequestArgs) -> u128 {
        // let mut dri: DataRequest = self.data_requests.get(payload.id.into()).expect("No dri with such id");
        // // Challenge answer should be valid in relation to the initial data request
        // assert!(dri.validate_answer(&payload.answer), "invalid answer");


        // let round: DataRequestRound = dri.rounds.iter().last().unwrap();
        // // Get the latest answer on the proposal, challenge answer should differ from the latest answer
        // assert!(round.winning_outcome().unwrap() != payload.answer, "EQ_CHALLENGE");

        // // Only continue if the last answer is challengeable
        // assert!(round.quorum_date > 0, "No quorum on previos round");
        // assert!(env::block_timestamp() < round.quorum_date + round.challenge_period, "Challenge period expired");

        // // Add new challenge
        // let mut outcomes =  HashSet::new();
        // outcomes.insert(payload.answer);
        // dri.rounds.push(&DataRequestRound {
        //     initiator: env::predecessor_account_id(),

        //     total: 0,
        //     outcomes,
        //     outcome_stakes: HashMap::default(),
        //     //users: HashMap::default(),
        //     user_outcome_stake: HashMap::default(),

        //     quorum_amount: 0, // todo calculate
        //     start_date: env::block_timestamp(),
        //     quorum_date: 0,
        //     challenge_period: 0// todo challenge_period
        // });
        // TODO: return unused stake
        0
    }

}