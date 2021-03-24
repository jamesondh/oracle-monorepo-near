use crate::*;
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::json_types::{ U64 };
use near_sdk::serde::{ Deserialize, Serialize };
use near_sdk::{ env, Balance, AccountId };
use near_sdk::collections::{ Vector, LookupMap };

use crate::types::{ Timestamp };

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
pub enum Outcome {
    Answer(String),
    Invalid
}

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
pub struct Source {
    pub end_point: Vec<String>,
    pub source_path: Vec<String>
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ResolutionWindow {
    pub close_time: Timestamp,
    pub outcome_to_stake: LookupMap<String, Balance>,
    pub user_to_outcome_to_stake: LookupMap<AccountId, LookupMap<String, Balance>>,
    pub bonded_outcome: Option<Outcome>
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct DataRequest {
    pub sources: Vec<String>,
    pub source_path: Vec<Source>,
    pub settlement_time: Timestamp,
    pub outcomes: Option<Vec<String>>,
    pub tvl_address: AccountId,
    pub tvl_function: String,
    pub validity_bond: u128,
    pub finalized_outcome: Option<Outcome>,
    pub resolution_windows: Vector<ResolutionWindow>
}

impl DataRequest {
    fn get_final_outcome(&self) -> Option<Outcome> {
        let last_bonded_window_i = self.resolution_windows.len() - 2; // Last window after end_time never has a bonded outcome
        let last_bonded_window = self.resolution_windows.get(last_bonded_window_i).unwrap();
        last_bonded_window.bonded_outcome
    }

    fn can_finalize(&self) -> bool {
        let last_window = self.resolution_windows.iter().last().expect("No resolutions found, DataRequest not processed");
        if env::block_timestamp() >= last_window.close_time {
            true
        } else {
            false
        }
    }

    pub fn finalize(&mut self) {
        self.finalized_outcome = self.get_final_outcome();
    }
}

trait DataRequestHandler {
    fn dr_finalize(&mut self, id: U64);
}

impl DataRequestHandler for Contract {
    fn dr_finalize(&mut self, id: U64) {
        let mut dr = self.dr_get_expect(id);
        assert!(dr.finalized_outcome.is_none(), "DataRequest already finalized");
        assert!(dr.can_finalize(), "Challenge window still open");
        dr.finalize();
        self.data_requests.replace(id.into(), &dr);
    }
}

impl Contract {
    fn dr_get_expect(&self, id: U64) -> DataRequest {
        self.data_requests.get(id.into()).expect("DataRequest with this id does not exist")
    }
}
