use std::collections::HashMap;

use near_sdk::{ ext_contract, AccountId, Balance, Gas, env, near_bindgen, Promise, PromiseOrValue};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::collections::{UnorderedSet, Vector, UnorderedMap};
use near_sdk::json_types::{U64, U128};

const TX_GAS: u64 = 5_000_000_000_000;

#[ext_contract]
pub trait NEP21 {
    fn set_allowance(&mut self, escrow_account_id: AccountId, allowance: U128);
    fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, amount: U128);
    fn transfer(&mut self, new_owner_id: AccountId, amount: U128);
    fn get_total_supply(&self) -> U128;
    fn get_balance(&self, owner_id: AccountId) -> U128;
    fn get_allowance(&self, owner_id: AccountId, escrow_account_id: AccountId) -> U128;
}

/// Policy item, defining how many votes required to approve up to this much amount.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FLX {
    pub address: AccountId,
}


impl FLX {
    pub fn transfer(&mut self, new_owner_id: AccountId, amount: u128) -> Promise {
        nep21::transfer(
            new_owner_id,
            U128(amount),
            &self.address,
            0,
            TX_GAS,
        )
    }

    pub fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, amount: u128) -> Promise {
        nep21::transfer_from(
            owner_id,
            new_owner_id,
            U128(amount),
            &self.address,
            0,
            TX_GAS,
        )
    }

    pub fn get_balance(&self, owner_id: AccountId) -> U128 {
        // let balance : U128 = nep21::get_balance(
        //     owner_id,
        //     &self.address,
        //     0,
        //     TX_GAS,
        // ).unwrap_json();
        // balance
        U128(100)
    }
}