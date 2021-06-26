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
        config: OracleConfig
    ) -> Self {        
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