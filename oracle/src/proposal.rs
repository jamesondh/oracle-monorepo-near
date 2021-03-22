use std::collections::HashMap;
use std::collections::HashSet;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{ AccountId, Balance, env };
use near_sdk::{ json_types::{U64, U128} };
use near_sdk::collections::{UnorderedSet, Vector, UnorderedMap};

use crate::vote_types::{ WrappedBalance, WrappedDuration, Duration, Vote, Timestamp };
use crate::policy_item::{ PolicyItem };
use crate::proposal_status::{ ProposalStatus };

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalInput {
    pub description: String,
    pub kind: ProposalKind,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RegistryEntry {
    pub interface_name: String,
    pub contract_entry: AccountId,
    pub callback: String,
    pub tvs_method: String,
    pub rvs_method: String,
    pub code_base_url: String
}

impl DataRequestRound {
    pub fn winning_outcome(&self) -> Option<String> {
        if self.outcomes.len() == 0 {
            None
        } else {
            let mut winning_outcome_answer : String = "".to_string();
            let mut winning_outcome_value : u128 = 0;

            for outcome in &self.outcomes {
                let value : &u128 = self.outcome_stakes.get(outcome).unwrap();

                // todo, equal?
                if *value > winning_outcome_value {
                    winning_outcome_answer = outcome.to_string();
                }
            }

            Some(winning_outcome_answer)
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DataRequestRound {
    pub initiator: AccountId,
    // context
    pub start_date: Timestamp,
    pub quorum_date: Timestamp,
    pub challenge_period: Duration,
    pub quorum_amount: u128,

    // // stakes
    pub total: u128,
    pub outcomes: HashSet<String>,
    pub outcome_stakes: HashMap<String, u128>,
    pub user_outcome_stake: HashMap<AccountId, HashMap<String, u128>>
}


#[derive(BorshSerialize, BorshDeserialize)]
//#[serde(crate = "near_sdk::serde")]
pub struct DataRequestInitiation {
    pub extra_info: Option<String>,
    pub source: String,
    pub majority_outcome: Option<String>,
    pub outcomes: Option<Vec<String>>,
    pub tvl_address: AccountId,
    pub tvl_function: String,
    pub rounds: Vector<DataRequestRound>,
    pub validity_bond: u128,
    pub finalized_at: Timestamp
}

impl DataRequestInitiation {
    pub fn validate_answer(&self, answer: &String) -> bool {
        match &self.outcomes {
            Some(v) => {
                v.contains(answer)
            },
            None => { true }
        }
    }
}


#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "type")]
pub enum ProposalKind {
    AddWhitelist(RegistryEntry),
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    pub status: ProposalStatus,
    pub proposer: AccountId,
    pub kind: ProposalKind,
    pub vote_period_end: Duration,
    pub vote_yes: u128,
    pub vote_no: u128,
    pub votes: HashMap<AccountId, Vote>,
    pub finalized_at: u64
}

impl Proposal {

    /// Compute new vote status given council size and current timestamp.
    pub fn vote_status(&self, policy: &PolicyItem, num_council: u64) -> ProposalStatus {
        // let needed_votes = policy.num_votes(num_council);

        // if self.vote_yes >= needed_votes {
        ProposalStatus::Success
        // } else if env::block_timestamp() < self.vote_period_end {
        //     ProposalStatus::Vote
        // } else {
        //     ProposalStatus::Reject
        // }
    }
}
