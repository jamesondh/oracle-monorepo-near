use crate::*;
use near_sdk::{ext_contract, Promise, PromiseOrValue, Gas};
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };

// TODO: figure out view call price
const GAS_BASE_TRANSFER: Gas = 5_000_000_000_000;

#[ext_contract]
pub trait ExtSelf {
    fn continue_tvs_calc(&self, 
        // sum: U128, next_account: std::option::Option<(std::string::String, whitelist::RegistryEntry)>
    ) -> Promise;
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
impl Contract {
    #[private]
    pub fn continue_tvs_calc(&self, 
        // sum: U128, next_account: Option<(std::string::String, whitelist::RegistryEntry)>
    ) -> U128 {
        0.into()
    }

    pub fn fetch_tvs(&self) -> Promise {
        let account = self.whitelist.0.iter().next();
        ext_requestor::get_tvs(
            &account.unwrap().1.contract_entry,
            0,
            GAS_BASE_TRANSFER
        )
        .then(
            ext_self::continue_tvs_calc(
                // U128(0),
                // self.whitelist.0.iter().next(),
                &env::current_account_id(),
                0,
                GAS_BASE_TRANSFER
            )
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

    #[test]
    fn fetch_tvs() {
        testing_env!(get_context(carol()));
        let whitelist = Some(vec![registry_entry(bob()), registry_entry(carol())]);
        let contract = Contract::new(whitelist, config());
        let tvs = contract.fetch_tvs();
    }
}
