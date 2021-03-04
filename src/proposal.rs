use std::collections::HashMap;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{ AccountId, Balance, env };
use near_sdk::{ json_types::{U64, U128} };
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

// description: String,
// extra_info: Option<String>,
// source: String,
// outcomes: Option<Vec<String>>,
// settlement_date: Timestamp,
// challenge_period: Timestamp,
// settlement_bond_address: AccountId,
// settlement_cb: String

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DataRequestStake {
    pub total: u128,
    pub outcomes: HashMap<String, u128>,
    pub users: HashMap<AccountId, u128>,
    pub users_outcomes: HashMap<AccountId, String>
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DataRequestChallenge {
    pub initiator: AccountId,
    pub extra_info: Option<String>,
    pub source: String,
    pub outcomes: Option<Vec<String>>,
    pub settlement_date: Timestamp,
    pub challenge_period: Timestamp,
    pub tvl: u128,
    pub tvl_address: AccountId,
    pub tvl_function: String,
    pub stakes: DataRequestStake
}


#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DataRequestInitiation {
    pub initiator: AccountId,
    pub extra_info: Option<String>,
    pub source: String,
    pub outcomes: Option<Vec<String>>,
    pub settlement_date: Timestamp,
    pub challenge_period: Timestamp,
    pub tvl: u128,
    pub tvl_address: AccountId,
    pub tvl_function: String,
    pub stakes: DataRequestStake
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "type")]
pub enum ProposalKind {
    AddWhitelist { target: RegistryEntry },
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
