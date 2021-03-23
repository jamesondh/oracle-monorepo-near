use crate::*;
use near_sdk::serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize)]
pub struct NewDataRequestArgs {
    pub description: String,
    pub extra_info: String,
    pub source: String,
    pub outcomes: Option<Vec<String>>,
    pub settlement_date: Timestamp,
    pub challenge_period: Timestamp,
    pub tvl_address: AccountId,
    pub tvl_function: String,
}

#[derive(Serialize, Deserialize)]
pub struct StakeDataRequestArgs {
    pub id: U64,
    pub answer: String,
}