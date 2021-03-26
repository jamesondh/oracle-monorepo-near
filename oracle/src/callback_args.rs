use crate::*;
use near_sdk::serde::{ Serialize, Deserialize };

const MAX_SOURCES: u8 = 8;
const MIN_OUTCOMES: u8 = 2;
const MIN_PERIOD_MULTIPLIER: u64 = 3;
const MAX_SETTLEMENT_DURATION: Duration = 1_000_000_000 * 60 * 60 * 24 * 365 * 3; // ~3 years

#[derive(Serialize, Deserialize)]
pub struct NewDataRequestArgs {
    pub sources: Vec<data_request::Source>,
    pub outcomes: Option<Vec<String>>,
    pub settlement_time: Timestamp, // Can be in the past
    pub challenge_period: Timestamp,
    pub target_contract: AccountId
}

impl Contract {
    pub fn dr_validate(&self, data_request: &NewDataRequestArgs) {
        assert!(data_request.sources.len() as u8 <= MAX_SOURCES, "Too many sources provided, max sources is: {}", MAX_SOURCES);
        assert!(data_request.challenge_period >= self.config.min_initial_challenge_window_duration, "Challenge shorter than minimum challenge period of {}", self.config.min_initial_challenge_window_duration);
        assert!(data_request.challenge_period <= self.config.default_challenge_window_duration * MIN_PERIOD_MULTIPLIER, "Challenge period exceeds maximum challenge period of {}", self.config.default_challenge_window_duration * 3);
        assert!(data_request.settlement_time < env::block_timestamp() + MAX_SETTLEMENT_DURATION, "Exceeds max duration");
        assert!(
            data_request.outcomes.is_none() ||
            data_request.outcomes.as_ref().unwrap().len() as u8 <= self.config.max_outcomes &&
            data_request.outcomes.as_ref().unwrap().len() as u8 >= MIN_OUTCOMES,
            "Invalid outcome list either exceeds min of: {} or max of {}",
            MIN_OUTCOMES,
            self.config.max_outcomes
        );
    }
}

#[derive(Serialize, Deserialize)]
pub struct StakeDataRequestArgs {
    pub id: U64,
    pub outcome: data_request::Outcome,
}

#[derive(Serialize, Deserialize)]
pub struct ChallengeDataRequestArgs {
    pub id: U64,
    pub answer: data_request::Outcome,
}