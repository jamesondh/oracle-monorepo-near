use crate::utils::*;
pub struct TargetContractUtils {
    pub contract: ContractAccount<TargetContract>
}

impl TargetContractUtils {
    pub fn new(master_account: &TestAccount) -> Self {
        // deploy token
        let contract = deploy!(
            // Contract Proxy
            contract: TargetContract,
            // Contract account id
            contract_id: TARGET_CONTRACT_ID,
            // Bytes of contract
            bytes: &TARGET_CONTRACT_WASM_BYTES,
            // User deploying the contract,
            signer_account: master_account.account,
            deposit: to_yocto("1000"),
            // init method
            init_method: new(
                ORACLE_CONTRACT_ID.to_string()
            )
        );

        Self {
            contract
        }
    }
}