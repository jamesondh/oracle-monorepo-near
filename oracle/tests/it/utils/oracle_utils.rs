use crate::utils::*;
use oracle::oracle_config::OracleConfig;
use oracle::whitelist::{RegistryEntry, CustomFeeStakeArgs};

pub struct OracleUtils {
    pub contract: ContractAccount<OracleContract>
}

fn new_registry_entry(contract_id: String, custom_fee: CustomFeeStakeArgs) -> RegistryEntry {
    RegistryEntry {
        code_base_url: None,
        contract_entry: contract_id,
        interface_name: "test".to_string(),
        custom_fee
    }
}

impl OracleUtils {
    pub fn new(
        master_account: &TestAccount,
        custom_fee: CustomFeeStakeArgs,
        validity_bond: u128,
        final_arbitrator_invoke_amount: u128
    ) -> Self {        
        let config = OracleConfig {
            gov: "alice".to_string(),
            final_arbitrator: "alice".to_string(),
            bond_token: TOKEN_CONTRACT_ID.to_string(),
            stake_token: TOKEN_CONTRACT_ID.to_string(),
            validity_bond: U128(validity_bond),
            max_outcomes: 8,
            default_challenge_window_duration: U64(1000),
            min_initial_challenge_window_duration: U64(1000),
            // final_arbitrator_invoke_amount: U128(2500),
            final_arbitrator_invoke_amount: U128(final_arbitrator_invoke_amount),
            resolution_fee_percentage: 10_000,
        };
        
        // deploy token
        let contract = deploy!(
            // Contract Proxy
            contract: OracleContract,
            // Contract account id
            contract_id: ORACLE_CONTRACT_ID,
            // Bytes of contract
            bytes: &ORACLE_WASM_BYTES,
            // User deploying the contract,
            signer_account: master_account.account,
            deposit: to_yocto("1000"),
            // init method
            init_method: new(
                Some(vec![
                        new_registry_entry(REQUEST_INTERFACE_CONTRACT_ID.to_string(), custom_fee), 
                        new_registry_entry(TARGET_CONTRACT_ID.to_string(), CustomFeeStakeArgs::None)
                    ]
                ), 
                config
            )
        );

        storage_deposit(TOKEN_CONTRACT_ID, &master_account.account, SAFE_STORAGE_AMOUNT, Some(ORACLE_CONTRACT_ID.to_string()));


        Self {
            contract
        }
    }
}