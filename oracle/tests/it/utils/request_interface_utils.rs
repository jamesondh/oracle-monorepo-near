use crate::utils::*;
pub struct RequestInterfaceUtils {
    pub contract: ContractAccount<RequestInterfaceContract>
}

impl RequestInterfaceUtils {
    pub fn new(master_account: &TestAccount) -> Self {
        // deploy token
        let contract = deploy!(
            // Contract Proxy
            contract: RequestInterfaceContract,
            // Contract account id
            contract_id: REQUEST_INTERFACE_CONTRACT_ID,
            // Bytes of contract
            bytes: &REQUEST_INTERFACE_WASM_BYTES,
            // User deploying the contract,
            signer_account: master_account.account,
            deposit: to_yocto("1000"),
            // init method
            init_method: new(
                ORACLE_CONTRACT_ID.to_string(),
                TOKEN_CONTRACT_ID.to_string()
            )
        );

        Self {
            contract
        }
    }
}