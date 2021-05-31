use crate::utils::*;
pub struct TokenUtils {
    pub contract: ContractAccount<TokenContract>
}

impl TokenUtils {
    pub fn new(master_account: &TestAccount) -> Self {
        // deploy token
        let contract = deploy!(
            // Contract Proxy
            contract: TokenContract,
            // Contract account id
            contract_id: TOKEN_CONTRACT_ID,
            // Bytes of contract
            bytes: &TOKEN_WASM_BYTES,
            // User deploying the contract,
            signer_account: master_account.account,
            deposit: to_yocto("1000"),
            // init method
            init_method: new_default_meta(
                "alice".try_into().expect("invalid_account_id"),
                U128(to_yocto("1000000000"))
            )
        );

        Self {
            contract
        }
    }
}