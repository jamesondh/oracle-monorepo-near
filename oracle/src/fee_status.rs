use crate::*;
use near_sdk::{ext_contract, Promise, PromiseOrValue, Gas, PromiseResult};
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };

// TODO: figure out view call price
const GAS_BASE_TRANSFER: Gas = 5_000_000_000_000;

#[ext_contract]
pub trait ExtSelf {
    fn proceed_tvs_calc(&self, sum: U128, values: Vec<RegistryEntry>, index: u16) -> Promise;
}

#[ext_contract]
pub trait ExtRequestor {
    fn get_tvs(&self) -> Promise;
}

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

#[near_bindgen]
// TODO: fix gas pricing for these view calls
impl Contract {
    #[private]
    pub fn proceed_tvs_calc(&self, sum: U128, values: Vec<RegistryEntry>, index: u16) -> PromiseOrValue<U128> {
        let sum: u128 = sum.into();

        env::log(format!("doing this {}", index).as_bytes());
        let prev_tvs: u128 = if index > 0 {
             match env::promise_result(0) {
                PromiseResult::NotReady => panic!("something went wrong while fetching tvs"),
                PromiseResult::Successful(value) => {
                    if let Ok(tvs) = near_sdk::serde_json::from_slice::<U128>(&value) {
                        tvs.0
                    } else {
                        0
                    }
                }
                PromiseResult::Failed => 0,
            }
        } else {
            0
        };

        let new_sum = sum + prev_tvs;

        if index as usize >= values.len() {
            return PromiseOrValue::Value(new_sum.into())
        }

        match values.get(index as usize) {
            Some(val) => {
                PromiseOrValue::Promise(
                    ext_requestor::get_tvs(
                        &val.contract_entry,
                        0,
                        GAS_BASE_TRANSFER
                    )
                    .then(
                        ext_self::proceed_tvs_calc(
                            U128(new_sum),
                            values,
                            index + 1,
                            &env::current_account_id(),
                            0,
                            GAS_BASE_TRANSFER
                        )
                    )
                )
            }, 
            None => return PromiseOrValue::Value(0.into())
        }
    }

    pub fn fetch_tvs(&self) -> Promise {
        let values: Vec<RegistryEntry> = self.whitelist.0.values().collect();

        ext_self::proceed_tvs_calc(
            U128(0),
            values,
            0,
            &env::current_account_id(),
            0,
            GAS_BASE_TRANSFER
        )
    }
    
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod mock_token_basic_tests {
    use near_sdk::{ MockedBlockchain };
    use near_sdk::{ testing_env, VMContext };
    use super::*;

    fn alice() -> AccountId {
        "alice.near".to_string()
    }

    fn bob() -> AccountId {
        "bob.near".to_string()
    }

    fn carol() -> AccountId {
        "carol.near".to_string()
    }

    fn token() -> AccountId {
        "token.near".to_string()
    }

    fn gov() -> AccountId {
        "gov.near".to_string()
    }

    fn registry_entry(account: AccountId) -> RegistryEntry {
        RegistryEntry {
            interface_name: account.clone(),
            contract_entry: account.clone(),
            code_base_url: None
        }
    }

    fn config() -> oracle_config::OracleConfig {
        oracle_config::OracleConfig {
            gov: gov(),
            final_arbitrator: alice(),
            bond_token: token(),
            stake_token: token(),
            validity_bond: U128(0),
            max_outcomes: 8,
            default_challenge_window_duration: U64(1000),
            min_initial_challenge_window_duration: U64(1000),
            final_arbitrator_invoke_amount: U128(25_000_000_000_000_000_000_000_000_000_000),
            resolution_fee_percentage: 0,
        }
    }

    fn get_context(predecessor_account_id: AccountId) -> VMContext {
        VMContext {
            current_account_id: token(),
            signer_account_id: bob(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 1000 * 10u128.pow(24),
            account_locked_balance: 0,
            storage_usage: 10u64.pow(6),
            attached_deposit: 1000 * 10u128.pow(24),
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }
}
