use crate::*;
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::json_types::{ U64 };
use near_sdk::serde::{ Deserialize, Serialize };
use near_sdk::{ env, Balance, AccountId };
use near_sdk::collections::{ Vector, LookupMap };

use crate::types::{ Timestamp, Duration };

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
pub enum Outcome {
    Answer(String),
    Invalid
}

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
pub struct Source {
    pub end_point: String,
    pub source_path: String
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
    pub sources: Vec<Source>,
    pub settlement_time: Timestamp,
    pub outcomes: Option<Vec<String>>,
    pub challenge_period: Duration,
    pub creator: AccountId,
    pub finalized_outcome: Option<Outcome>,
    pub resolution_windows: Vector<ResolutionWindow>
}

impl DataRequest {
    fn new(sender: AccountId, dr_id: u64, request_data: NewDataRequestArgs) -> Self {
        let resolution_windows = Vector::new(format!("rw{}", dr_id).as_bytes().to_vec());
        Self {
            sources: request_data.sources,
            settlement_time: request_data.settlement_time,
            outcomes: request_data.outcomes,
            challenge_period: request_data.challenge_period,
            creator: sender,
            finalized_outcome: None,
            resolution_windows
        } 
    }

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
        // TODO: Return validity bond to creator
    }
}

impl Contract {
    fn dr_get_expect(&self, id: U64) -> DataRequest {
        self.data_requests.get(id.into()).expect("DataRequest with this id does not exist")
    }
    
    pub (crate) fn dr_new(&mut self, sender: AccountId, amount: u128, payload: NewDataRequestArgs) -> u128 {
        self.assert_whitelisted(sender);
        self.assert_bond_token();
        self.dr_validate(payload);
        assert!(amount >= self.config.validity_bond);

        let dr = DataRequest::new(sender, self.data_requests.len() as u64, payload);
        self.data_requests.push(&dr);

        if amount > self.config.validity_bond {
            amount - self.config.validity_bond
        } else {
            0
        }
    }
}

