use crate::*;

use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::AccountId;
use near_sdk::collections::LookupMap;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct FeeStatus {
    pub market_cap: u128,
    pub total_value_secured: u128,
    pub fee_percentage: u16, // denominated in 1e5 100000 == 1 == 100% && 1 = 0.00001 == 0.001%
}

impl FeeStatus {
    pub fn new() -> Self {
        Self {
            market_cap: 0,
            total_value_secured: 0,
            fee_percentage: 1
        }
    }
}

impl Contract {
    pub fn fetch_tvs(&self) -> U128{
        0.into()
    }
}
