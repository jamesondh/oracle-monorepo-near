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
    pub id: u64,
    pub sources: Vec<Source>,
    pub settlement_time: Timestamp,
    pub outcomes: Option<Vec<String>>,
    pub challenge_period: Duration,
    pub requestor: mock_requestor::Requestor,
    pub finalized_outcome: Option<Outcome>,
    pub resolution_windows: Vector<ResolutionWindow>
}

impl DataRequest {
    fn new(sender: AccountId, id: u64, request_data: NewDataRequestArgs) -> Self {
        let resolution_windows = Vector::new(format!("rw{}", id).as_bytes().to_vec());
        Self {
            id: id,
            sources: request_data.sources,
            settlement_time: request_data.settlement_time,
            outcomes: request_data.outcomes,
            challenge_period: request_data.challenge_period,
            requestor: mock_requestor::Requestor(sender),
            finalized_outcome: None,
            resolution_windows
        } 
    }

    pub fn finalize(&mut self) {
        self.finalized_outcome = self.get_final_outcome();
    }
}

impl DataRequest {
    fn assert_valid_answer(&self, answer: Outcome) {
        match self.outcomes {
            Some(outcomes) => match answer {
                Outcome::Answer(answer) => assert!(outcomes.contains(&answer), "Incompatible answer"),
                Invalid => ()
            },
            None => ()
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

    fn get_tvl(&self) -> u128 {
        self.requestor.get_tvl(self.id.into()).into()
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
    
    pub fn dr_new(&mut self, sender: AccountId, amount: u128, payload: NewDataRequestArgs) -> u128 {
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
    pub fn dr_stake(&mut self, sender: AccountId, amount: u128, payload: StakeDataRequestArgs) -> u128 {
        
        // assert!(dri.validate_answer(&payload.answer), "invalid answer");
        let dr = self.dr_get_expect(payload.id.into());
        dr.assert_valid_answer(payload.answer);
        let _tvl = dr.get_tvl(); // TODO: replace existing tvl logic

        // let mut round: DataRequestRound = dri.rounds.iter().last().unwrap();
        // if dri.rounds.len() > 1 {
        //     assert!(*round.outcomes.iter().next().unwrap() == payload.answer);
        // }
        // assert!(round.start_date > env::block_timestamp(), "NOT STARTED");
        // assert!(round.quorum_date == 0, "ALREADY PASSED");

        // round.total += amount;

        // let new_outcome_stake: u128 = match round.outcome_stakes.get_mut(&payload.answer) {
        //     Some(v) => {
        //         *v += amount;
        //         *v
        //     },
        //     None => {
        //         round.outcome_stakes.insert(payload.answer.clone(), amount);
        //         amount
        //     }
        // };
        // if new_outcome_stake > round.quorum_amount {
        //     round.quorum_date = env::block_timestamp();
        // }

        // let user_entries: &mut HashMap<String, u128> = match round.user_outcome_stake.get_mut(&env::predecessor_account_id()) {
        //     Some(v) => {
        //         v
        //     }
        //     None => {
        //         round.user_outcome_stake.insert(env::predecessor_account_id(), HashMap::default());
        //         round.user_outcome_stake.get_mut(&env::predecessor_account_id()).unwrap()
        //     }
        // };

        // match user_entries.get_mut(&payload.answer) {
        //     Some(v) => {
        //         *v += amount;
        //     }
        //     None => {
        //         user_entries.insert(payload.answer, amount);
        //     }
        // }

        // TODO: return unspent tokens
        0
    }

    
}

