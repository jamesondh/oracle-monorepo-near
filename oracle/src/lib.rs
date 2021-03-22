#![allow(clippy::too_many_arguments)]
use std::collections::HashMap;
use std::collections::HashSet;

use near_sdk::{ AccountId, Balance, Gas, env, near_bindgen, Promise, PromiseOrValue };
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedSet, Vector, UnorderedMap};
use near_sdk::json_types::{U64, U128};

mod proposal_status;
mod policy_item;
mod vote_types;
mod proposal;
mod mock_token;

pub use proposal::{ Proposal, ProposalInput, ProposalKind, RegistryEntry, DataRequestInitiation, DataRequestRound };
pub use proposal_status::{ ProposalStatus };
use vote_types::{ Duration, WrappedDuration, Vote, Timestamp };


#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize )]
pub struct FluxOracle {
    pub whitelist: HashMap<AccountId, RegistryEntry>,
    pub whitelist_proposals: Vector<Proposal>,
    pub whitelist_grace_period: u64,

    pub dri_registry: Vector<DataRequestInitiation>,

    pub proposal_bond: u128,
    pub validity_bond: u128,
    pub min_voters: u128,
    pub min_voters_agree: u128,
    pub token: flux_token::FLX,
    pub vote_period: Duration
}

impl Default for FluxOracle {
    fn default() -> Self {
        env::panic(b"FluxOracle should be initialized before usage")
    }
}

#[near_bindgen]
impl FluxOracle {
    #[init]
    pub fn new(
        address: AccountId,
        vote_period: WrappedDuration
    ) -> Self {
        Self {
            whitelist: HashMap::default(),
            whitelist_proposals: Vector::new(b"p".to_vec()),
            whitelist_grace_period: 1,

            dri_registry: Vector::new(b"r".to_vec()),

            proposal_bond: 1,
            validity_bond: 1,
            min_voters: 0,
            min_voters_agree: 1,
            token: flux_token::FLX{address},
            vote_period: vote_period.into()
        }
    }


    // rest of arguments? Or just proposal struct as argument
    // pub fn whitelist_proposal(&mut self, proposal: Proposal)
    pub fn whitelist_proposal(&mut self,
        contract_entry: AccountId,
        interface_name: String,
        callback: String,
        tvs_method: String,
        rvs_method: String,
        code_base_url: String
    ) -> U64 {
        // TODO
        // assert fields (e.g. non empty string)

        // TODO
        // do we want to link proposals to proposal_bond?
        // bij het updaten van proposal_bond zal dan de orginele proposal worden returned

        // TODO
        // Implement receiver method (instead of transfer from)
        // https://github.com/near/core-contracts/blob/w-near-141/w-near-141/src/fungible_token_core.rs#L81
        self.token.transfer_from(env::predecessor_account_id(), env::current_account_id(), self.proposal_bond);

        let registry_entry = RegistryEntry {
            interface_name,
            contract_entry,
            callback,
            tvs_method,
            rvs_method,
            code_base_url
        };

        let p = Proposal {
            status: ProposalStatus::Vote,
            proposer: env::predecessor_account_id(),
            kind: ProposalKind::AddWhitelist(registry_entry),
            vote_period_end: env::block_timestamp() + self.vote_period,
            vote_yes: 0,
            vote_no: 0,
            votes: HashMap::default(),
            finalized_at: 0
        };

        self.whitelist_proposals.push(&p);
        U64(self.whitelist_proposals.len() - 1)
    }

    pub fn whitelist_vote(&mut self, whitelist_proposal_id: U64, vote: Vote) {
        let mut proposal = self.whitelist_proposals.get(whitelist_proposal_id.into()).expect("No proposal with such id");
        assert_eq!(
            proposal.status,
            ProposalStatus::Vote,
            "Proposal not active voting"
        );
        assert!(proposal.vote_period_end <= env::block_timestamp(), "timestamp");

        // TODO
        // Implement receiver method (instead of transfer from)
        // https://github.com/near/core-contracts/blob/w-near-141/w-near-141/src/fungible_token_core.rs#L81
        let weight : u128 = self.token.get_balance(env::predecessor_account_id()).into();
        match vote {
            Vote::Yes => proposal.vote_yes += weight,
            Vote::No => proposal.vote_no += weight,
        }
        proposal.votes.insert(env::predecessor_account_id(), vote);
        // TODO just don;t update status?
        //proposal.status = proposal.vote_status(&self.policy, self.council.len());
        self.whitelist_proposals.replace(whitelist_proposal_id.into(), &proposal);
    }

    pub fn whitelist_finalize(&mut self, whitelist_proposal_id: U64) {
        let mut proposal = self.whitelist_proposals.get(whitelist_proposal_id.into()).expect("No proposal with such id");
        assert_eq!(
            proposal.status,
            ProposalStatus::Vote,
            "Proposal not active voting"
        );

        // Does at least 10% of flux tokens need to vote and 70% should be yes? ( so 7% on yes )
        // or does at least 10% of flux tokens need to vote on yes? ( + keep 70% ratio )

        //assert proposal.vote_yes > self.min_voters
        //assert proposal.min_voters_agree
        // finalize
        // else
        // if timestamp expired
        // reject
        // else
        // env::panic()
        proposal.status = ProposalStatus::Success;
        proposal.finalized_at = env::block_timestamp();
        self.whitelist_proposals.replace(whitelist_proposal_id.into(), &proposal);
    }

    pub fn whitelist_execute(&mut self,  whitelist_proposal_id: U64) {
        let proposal = self.whitelist_proposals.get(whitelist_proposal_id.into()).expect("No proposal with such id");
        assert_eq!(
            proposal.status,
            ProposalStatus::Success,
            "Proposal not success"
        );
        assert!(proposal.finalized_at + self.whitelist_grace_period <= env::block_timestamp(), "grace period");

        match proposal.kind {
            ProposalKind::AddWhitelist(target) => {
                self.whitelist.insert(target.contract_entry.clone(), target);
            }
        }
    }

    pub fn data_request_initiation(&mut self,
        _description: String, // TODO: log
        extra_info: Option<String>,
        source: String,
        outcomes: Option<Vec<String>>,
        settlement_date: Timestamp,
        challenge_period: Timestamp,
        tvl_address: AccountId,
        tvl_function: String
    ) {
        if !self.whitelist.contains_key(&env::predecessor_account_id()) {
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
        let mut dri = DataRequestInitiation {
            extra_info,
            source,
            outcomes,
            majority_outcome: None,
            tvl_address,
            tvl_function,
            rounds: Vector::new(b"r".to_vec()),

            validity_bond: self.validity_bond,
            finalized_at: 0
        };
        dri.rounds.push(&DataRequestRound {
            initiator: env::predecessor_account_id(),

            total: 0,
            outcomes: HashSet::default(),
            outcome_stakes: HashMap::default(),
            user_outcome_stake: HashMap::default(),

            quorum_amount: 0,
            start_date: settlement_date,
            quorum_date: 0,
            challenge_period
        });
        self.dri_registry.push(&dri);
    }

    // @returns `tvl` = `total value locked` behind this DataRequest
    fn _data_request_tvl(&mut self, id: U64) -> bool {
        let dri : DataRequestInitiation = self.dri_registry.get(id.into()).expect("No dri with such id");
        if dri.rounds.get(0).unwrap().quorum_amount != 0 {
            return false;
        }
        // calculate tvl by dri.tvl_address, dri.tvl_function
        // assert tvl > 0

        dri.rounds.get(0).unwrap().quorum_amount = 5;
        true
    }

    pub fn data_request_tvl(&mut self, id: U64) {
        assert!(self._data_request_tvl(id), "FAILED");
    }

    pub fn data_request_finalize(&mut self, id: U64) {
        let mut dri: DataRequestInitiation = self.dri_registry.get(id.into()).expect("No dri with such id");

        let current_round: DataRequestRound = dri.rounds.iter().last().unwrap();
        dri.majority_outcome = current_round.winning_outcome();
        dri.finalized_at = env::block_timestamp();
        assert!(current_round.quorum_date > 0, "QUORUM NOT REACHED");
        assert!(env::block_timestamp() > current_round.quorum_date + current_round.challenge_period, "CHALLENGE_PERIOD_ACTIVE");
    }

    pub fn data_request_finalize_claim(&mut self, id: U64) {
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
    pub fn data_request_stake(&mut self, id: U64, answer: String) {
        let dri : DataRequestInitiation = self.dri_registry.get(id.into()).expect("No dri with such id");
        assert!(dri.validate_answer(&answer), "invalid answer");
        self._data_request_tvl(id);

        let mut round : DataRequestRound = dri.rounds.iter().last().unwrap();
        if dri.rounds.len() > 1 {
            assert!(*round.outcomes.iter().next().unwrap() == answer);
        }
        assert!(round.start_date > env::block_timestamp(), "NOT STARTED");
        assert!(round.quorum_date == 0, "ALREADY PASSED");

        // TODO
        // receiving flux tokens
        let amount : u128 = 5;
        round.total += amount;

        let new_outcome_stake : u128 = match round.outcome_stakes.get_mut(&answer) {
            Some(v) => {
                *v += amount;
                *v
            },
            None => {
                round.outcome_stakes.insert(answer.clone(), amount);
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

        match user_entries.get_mut(&answer) {
            Some(v) => {
                *v += amount;
            }
            None => {
                user_entries.insert(answer, amount);
            }
        }
    }

    pub fn data_request_challenge(&mut self, id: U64, answer: String) {
        let mut dri : DataRequestInitiation = self.dri_registry.get(id.into()).expect("No dri with such id");
        // Challenge answer should be valid in relation to the initial data request
        assert!(dri.validate_answer(&answer), "invalid answer");


        let round : DataRequestRound = dri.rounds.iter().last().unwrap();
        // Get the latest answer on the proposal, challenge answer should differ from the latest answer
        assert!(round.winning_outcome().unwrap() != answer, "EQ_CHALLENGE");

        // Only continue if the last answer is challengeable
        assert!(round.quorum_date > 0, "No quorum on previos round");
        assert!(env::block_timestamp() < round.quorum_date + round.challenge_period, "Challenge period expired");

        // Add new challenge
        let mut outcomes =  HashSet::new();
        outcomes.insert(answer);
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
        })
    }

}


// todo
// keep whitelist of account ids
// voting process