use crate::*;
use near_sdk::serde::{ Deserialize, Serialize };

const MAX_RESOLUTION_FEE_PERCENTAGE: u32 = 5000; // 5% in 1e5

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
pub struct FeeConfig {
    // total market cap of FLUX/stake_token denominated in bond_token
    pub flux_market_cap: U128,
    // total value staked (TVS) of all request interfaces; denominated in bond_token
    pub total_value_staked: U128,
    // global percentage of TVS to pay out to resolutors; denominated in 1e5 so 1 = 0.001%, 100000 = 100%
    pub resolution_fee_percentage: u32,
}

#[near_bindgen]
impl Contract {
    // @notice sets FLUX market cap and TVS for fee calculation; callable only by council
    pub fn update_fee_config(
        &mut self,
        new_fee_config: FeeConfig,
    ) {
        self.assert_gov();

        assert!(
            u128::from(new_fee_config.total_value_staked) < u128::from(new_fee_config.flux_market_cap),
            "TVS must be lower than market cap"
        );
        assert!(
            new_fee_config.resolution_fee_percentage <= MAX_RESOLUTION_FEE_PERCENTAGE,
            "Exceeds max resolution fee percentage"
        );

        self.fee_config = new_fee_config.clone();

        logger::log_fee_config(&new_fee_config);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod mock_token_basic_tests {
    use near_sdk::{ MockedBlockchain };
    use near_sdk::{ testing_env, VMContext };
    use near_sdk::json_types::U128;
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
            validity_bond: U128(0),
            max_outcomes: 8,
            default_challenge_window_duration: U64(1000),
            min_initial_challenge_window_duration: U64(1000),
            final_arbitrator_invoke_amount: U128(25_000_000_000_000_000_000_000_000_000_000),
        }
    }

    fn fee_config() -> FeeConfig {
        FeeConfig {
            flux_market_cap: U128(50000),
            total_value_staked: U128(10000),
            resolution_fee_percentage: 5000, // 5%
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
            attached_deposit: 15600000000000000000000,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    fn g_update_fee() {
        testing_env!(get_context(gov()));
        let mut contract = Contract::new(None, config(gov()), fee_config());
        let new_fee_config = FeeConfig {
            flux_market_cap: U128(1234),
            total_value_staked: U128(123),
            resolution_fee_percentage: 999, // .999%
        };
        contract.update_fee_config(new_fee_config);
    }
    
    #[test]
    #[should_panic(expected = "This method is only callable by the governance contract gov.near")]
    fn g_update_fee_invalid() {
        testing_env!(get_context(gov()));
        let mut contract = Contract::new(None, config(gov()), fee_config());
        testing_env!(get_context(bob()));
        let new_fee_config = FeeConfig {
            flux_market_cap: U128(1234),
            total_value_staked: U128(123),
            resolution_fee_percentage: 999, // .999%
        };
        contract.update_fee_config(new_fee_config);
    }
}