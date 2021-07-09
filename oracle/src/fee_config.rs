use crate::*;

const MIN_RESOLUTION_FEE_PERCENTAGE: u16 = 1; // 0.001%
const MAX_RESOLUTION_FEE_PERCENTAGE: u16 = 1000; // 1%

pub struct FeeConfig {
    // total market cap of FLUX/stake_token denominated in bond_token
    pub flux_market_cap: WrappedBalance,
    // total value staked (TVS) of all request interfaces; denominated in stake_token/FLUX
    pub total_value_staked: WrappedBalance,
    // global percentage of TVS to pay out to resolutors; denominated in 1e5 so 1 = 0.001%, 100000 = 100%
    pub resolution_fee_percentage: u16,
}

#[near_bindgen]
impl Contract {
    // @notice sets FLUX market cap and TVS for fee calculation; callable only by council
    pub fn update_fee_config(
        &mut self,
        new_market_cap: WrappedBalance,
        new_tvs: WrappedBalance,
    ) {
        self.assert_gov();
        
        // TODO: calculate resolution fee percentage, aiming for TVS = 1/5 FLUX market cap
        let new_fee_percentage = MAX_RESOLUTION_FEE_PERCENTAGE;
        assert!(new_fee_percentage <= MAX_RESOLUTION_FEE_PERCENTAGE, "Exceeds max resolution fee percentage");

        let new_fee_config = FeeConfig {
            flux_market_cap: new_market_cap,
            total_value_staked: new_tvs,
            resolution_fee_percentage: new_fee_percentage,
        };

        self.fee_config = new_fee_config;

        // TODO: log
        // logger::log_update_market_cap(new_market_cap);
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
            attached_deposit: 15600000000000000000000,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    fn g_set_market_cap() {
        testing_env!(get_context(gov()));
        let mut contract = Contract::new(None, config(gov()));
        contract.set_market_cap(U128(500));
    }
    
    #[test]
    #[should_panic(expected = "This method is only callable by the governance contract gov.near")]
    fn g_set_market_cap_invalid() {
        testing_env!(get_context(gov()));
        let mut contract = Contract::new(None, config(gov()));
        testing_env!(get_context(bob()));
        contract.set_market_cap(U128(500));
    }
}