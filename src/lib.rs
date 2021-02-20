use near_sdk::{ ext_contract, AccountId, Balance, Gas, env, near_bindgen, Promise, PromiseOrValue};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedSet, Vector, UnorderedMap};
use near_sdk::json_types::{U64, U128};

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct FluxOracle {

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

    ) -> Self {
        let mut oracle = Self {

        };
        oracle
    }

}