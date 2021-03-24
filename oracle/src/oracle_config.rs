use crate::*;
use crate::types::{ Duration };
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::serde::{ Serialize, Deserialize };
use near_sdk::{ AccountId };

const MAX_FINAL_ARBITRATOR_INVOKE_AMOUNT: u16 = 500; // 5%
const MIN_FINAL_ARBITRATOR_INVOKE_AMOUNT: u16 = 100; // 1%
const MAX_RESOLUTION_FEE_PERCENTAGE: u16 = 100; // 1%

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct OracleConfig {
    pub gov: AccountId,
    pub final_arbitrator: AccountId, // Invoked to have last say in `DataRequest`, this happens when the `challenge_bond` for a `DataRequest` is >= than `final_arbitrator_invoke_amount` / 100 % of the total supply
    pub stake_token: AccountId,
    pub bond_token: AccountId,
    pub validity_bond: u128,
    pub default_challenge_window_duration: Duration,
    pub initial_challenge_window_duration: Duration,
    pub final_arbitrator_invoke_amount: u16, // Percentage of total supply that needs to be staked in `ChallengeWindow` to invoke the final arbitrator, denominated in 1e4 so 1 = 0.01% - 10000 = 100%
    pub resolution_fee_percentage: u16, // Percentage of requesters `tvl` behind the request that's to be paid out to resolutors, denominated in 1e4 so 1 = 0.01% - 10000 = 100%
    pub challenge_exponent: u8, // Set to 2 for now, creating it as a config variable in case of requested change
}

impl OracleConfig {
    pub fn new() -> Self {
        Self {
            gov: "test".to_string(),
            final_arbitrator: "test".to_string(),
            bond_token: "test".to_string(),
            stake_token: "test".to_string(),
            validity_bond: 1,
            default_challenge_window_duration: 123,
            initial_challenge_window_duration: 321,
            final_arbitrator_invoke_amount: 250,
            resolution_fee_percentage: 0,
            challenge_exponent: 2,
        }
    }
}

trait ConfigHandler {
    // Getters
    fn get_config(&self) -> &OracleConfig;

    // Setters
    fn set_config(&mut self, new_config: OracleConfig);
}

impl ConfigHandler for Contract {

    fn get_config(&self) -> &OracleConfig {
        &self.config
    }

    fn set_config(&mut self, new_config: OracleConfig) {
        self.assert_gov();
        assert!(new_config.final_arbitrator_invoke_amount <= MAX_FINAL_ARBITRATOR_INVOKE_AMOUNT, "Invalid arbitrator invoke amount");
        assert!(new_config.final_arbitrator_invoke_amount >= MIN_FINAL_ARBITRATOR_INVOKE_AMOUNT, "Invalid arbitrator invoke amount");
        assert!(new_config.resolution_fee_percentage <= MAX_RESOLUTION_FEE_PERCENTAGE, "Fee cannot be higher than 33%");
        assert_eq!(new_config.challenge_exponent, self.config.challenge_exponent, "Exponent cannot be altered for time being");

        self.config = new_config;
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

    fn token() -> AccountId {
        "token.near".to_string()
    }
 
    fn gov() -> AccountId {
        "gov.near".to_string()
    }

    fn config(gov: AccountId) -> oracle_config::OracleConfig {
        oracle_config::OracleConfig {
            gov,
            final_arbitrator: alice(),
            bond_token: token(),
            stake_token: token(),
            validity_bond: 0,
            default_challenge_window_duration: 1000,
            initial_challenge_window_duration: 1000,
            final_arbitrator_invoke_amount: 250,
            resolution_fee_percentage: 0,
            challenge_exponent: 2,
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
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    fn set_config_from_gov() {
        testing_env!(get_context(gov()));
        let mut contract = Contract::new(None, config(gov()));
        contract.set_config(config(alice()));
        assert_eq!(contract.config.gov, alice());
    }

    #[test]
    #[should_panic(expected = "This method is only callable by the governance contract gov.near")]
    fn fail_set_config_from_user() {
        testing_env!(get_context(alice()));
        let mut contract = Contract::new(None, config(gov()));
        contract.set_config(config(alice()));
    }

    // TODO: add gov assertion tests
}