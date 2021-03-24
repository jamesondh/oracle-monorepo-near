#![allow(clippy::too_many_arguments)]
// TODO: Remove Rust native types
use std::collections::HashMap;
use std::collections::HashSet;

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
use data_request::{ DataRequestInitiation, DataRequestRound};

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize )]
pub struct Contract {
    pub whitelist: whitelist::Whitelist,
    pub config: oracle_config::OracleConfig,
    pub dri_registry: Vector<DataRequestInitiation>,
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
            dri_registry: Vector::new(b"r".to_vec()),
            validity_bond: 1.into(),
            token: mock_token::Token::default_new(),
        }
    }

    pub fn dr_finalize(&mut self, id: U64) {
        let mut dri: DataRequestInitiation = self.dri_registry.get(id.into()).expect("No dri with such id");

        let current_round: DataRequestRound = dri.rounds.iter().last().unwrap();
        dri.majority_outcome = current_round.winning_outcome();
        dri.finalized_at = env::block_timestamp();
        assert!(current_round.quorum_date > 0, "QUORUM NOT REACHED");
        assert!(env::block_timestamp() > current_round.quorum_date + current_round.challenge_period, "CHALLENGE_PERIOD_ACTIVE");
    }

    pub fn dr_claim(&mut self, id: U64) {
        // calculate the amount of tokens the user
        let dri : DataRequestInitiation = self.dri_registry.get(id.into()).expect("No dri with such id");
        assert!(dri.finalized_at != 0, "DataRequest is already finalized");

        let final_outcome : String = dri.majority_outcome.unwrap();
        let mut user_amount : u128 = 0;
        let mut rounds_share_validity = 0;

        let round_zero : DataRequestRound = dri.rounds.get(0).unwrap();
        // If the initial round was the right answer, receives full validity bond
        if round_zero.winning_outcome().unwrap() == final_outcome {
            // divide validity bond among round 0 stakers
            user_amount += dri.validity_bond * round_zero.user_outcome_stake.
                get(&env::predecessor_account_id()).unwrap().
                get(&final_outcome).unwrap() / round_zero.outcome_stakes.get(&final_outcome).unwrap();
        } else {
            // loop over all the other round and divide validity bond over round who where right
            for n in 1..dri.rounds.len() {
                if dri.rounds.get(n).unwrap().winning_outcome().unwrap() == final_outcome {
                    rounds_share_validity += 1;
                }
            }
        }

        for n in 1..dri.rounds.len() {
            let current_round : DataRequestRound = dri.rounds.get(n).unwrap();
            if current_round.winning_outcome().unwrap() != final_outcome {
                continue;
            }

            if rounds_share_validity > 0 {
                // share validity bond
                user_amount += dri.validity_bond * current_round.user_outcome_stake.
                    get(&env::predecessor_account_id()).unwrap().
                    get(&final_outcome).unwrap() / current_round.outcome_stakes.get(&final_outcome).unwrap() / rounds_share_validity;
            }

            let losing_round : DataRequestRound = dri.rounds.get(n).unwrap();
            // original winning stake
            user_amount += current_round.user_outcome_stake.
              get(&env::predecessor_account_id()).unwrap().
              get(&final_outcome).unwrap();

            // add losing stakes
            user_amount += user_amount * losing_round.outcome_stakes.get(
                &losing_round.winning_outcome().unwrap()
            ).unwrap() / current_round.total;
        }
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

    fn dr_tvl(&self, id: U64) -> u128 {
        // TODO: Get DataRequest
        // TODO: Get owner
        // TODO: Call get_tvl_for_request for owner
        self.get_tvl_for_request(id).into()
    }

    fn dr_new(&mut self, _sender: AccountId, _payload: NewDataRequestArgs) -> u128 {
        if !self.whitelist.contains(env::predecessor_account_id()) {
            env::panic(b"not whitelisted");
        }
        
        // TODO
        // validate fields

        // TODO
        // settlement callback is basically code injection
        // reentry / malicious behaviour needs to be taken care of

        // TODO
        // validate MIN < challenge_period < MAX

        // TODO
        // check if validity bond attached (USDC)
        // add validity bond amount to DRI storage

        // TODO: Should be done with DataRequest::new
        // let mut dri = DataRequestInitiation {
        //     extra_info,
        //     source,
        //     outcomes,
        //     majority_outcome: None,
        //     tvl_address,
        //     tvl_function,
        //     rounds: Vector::new(b"r".to_vec()),

        //     validity_bond: self.validity_bond,
        //     finalized_at: 0
        // };

        // TODO: Should be done in DataRequest::new
        // dri.rounds.push(&DataRequestRound {
        //     initiator: env::predecessor_account_id(),

        //     total: 0,
        //     outcomes: HashSet::default(),
        //     outcome_stakes: HashMap::default(),
        //     user_outcome_stake: HashMap::default(),

        //     quorum_amount: 0,
        //     start_date: settlement_date,
        //     quorum_date: 0,
        //     challenge_period
        // });
        // self.dri_registry.push(&dri);

        // TODO: return unspent tokens
        0
    }

    // Challenge answer is used for the following scenario
    //     e.g.
    //     t = 0, challenge X is active
    //     t = 1, user send challenger transaction
    //     t = 2, challenge X is resolved, challenge Y is active
    //     t = 3, user TX is processed (stakes on wrong answer)
    /// Users can stake for a data request once (or they should unstake if thats possible)
    /// If the DRI has any predefined outcomes, the answers should be one of the predefined ones
    /// If the DRI does not have predefined outcomes, users can vote on answers freely
    /// The total stake is tracked, this stake get's divided amoung stakers with the most populair answer on finalization
    fn dr_stake(&mut self, _sender: AccountId, amount: U128, payload: StakeDataRequestArgs) -> u128 {
        let amount: u128 = amount.into();
        let dri : DataRequestInitiation = self.dri_registry.get(payload.id.into()).expect("No dri with such id");
        assert!(dri.validate_answer(&payload.answer), "invalid answer");
        
        let _tvl = self.dr_tvl(payload.id); // TODO: replace existing tvl logic

        let mut round : DataRequestRound = dri.rounds.iter().last().unwrap();
        if dri.rounds.len() > 1 {
            assert!(*round.outcomes.iter().next().unwrap() == payload.answer);
        }
        assert!(round.start_date > env::block_timestamp(), "NOT STARTED");
        assert!(round.quorum_date == 0, "ALREADY PASSED");

        round.total += amount;

        let new_outcome_stake : u128 = match round.outcome_stakes.get_mut(&payload.answer) {
            Some(v) => {
                *v += amount;
                *v
            },
            None => {
                round.outcome_stakes.insert(payload.answer.clone(), amount);
                amount
            }
        };
        if new_outcome_stake > round.quorum_amount {
            round.quorum_date = env::block_timestamp();
        }

        let user_entries : &mut HashMap<String, u128> = match round.user_outcome_stake.get_mut(&env::predecessor_account_id()) {
            Some(v) => {
                v
            }
            None => {
                round.user_outcome_stake.insert(env::predecessor_account_id(), HashMap::default());
                round.user_outcome_stake.get_mut(&env::predecessor_account_id()).unwrap()
            }
        };

        match user_entries.get_mut(&payload.answer) {
            Some(v) => {
                *v += amount;
            }
            None => {
                user_entries.insert(payload.answer, amount);
            }
        }

        // TODO: return unspent tokens
        0
    }

    // TODO: Pass in round as a param for challenges to avoid race conditions
    // TODO: Consume and account for amount
    fn dr_challenge(&mut self, _sender: AccountId, _amount: U128, payload: ChallengeDataRequestArgs) -> u128 {
        let mut dri : DataRequestInitiation = self.dri_registry.get(payload.id.into()).expect("No dri with such id");
        // Challenge answer should be valid in relation to the initial data request
        assert!(dri.validate_answer(&payload.answer), "invalid answer");


        let round : DataRequestRound = dri.rounds.iter().last().unwrap();
        // Get the latest answer on the proposal, challenge answer should differ from the latest answer
        assert!(round.winning_outcome().unwrap() != payload.answer, "EQ_CHALLENGE");

        // Only continue if the last answer is challengeable
        assert!(round.quorum_date > 0, "No quorum on previos round");
        assert!(env::block_timestamp() < round.quorum_date + round.challenge_period, "Challenge period expired");

        // Add new challenge
        let mut outcomes =  HashSet::new();
        outcomes.insert(payload.answer);
        dri.rounds.push(&DataRequestRound {
            initiator: env::predecessor_account_id(),

            total: 0,
            outcomes,
            outcome_stakes: HashMap::default(),
            //users: HashMap::default(),
            user_outcome_stake: HashMap::default(),

            quorum_amount: 0, // todo calculate
            start_date: env::block_timestamp(),
            quorum_date: 0,
            challenge_period: 0// todo challenge_period
        });
        // TODO: return unused stake
        0
    }

}