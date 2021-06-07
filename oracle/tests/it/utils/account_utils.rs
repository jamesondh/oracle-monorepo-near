use crate::utils::*;

pub fn init_balance() -> u128 {
    to_yocto("1000")
}

pub struct TestAccount {
    pub account: UserAccount
}

impl TestAccount {
    pub fn new(
        master_account: Option<&UserAccount>, 
        account_id: Option<&str>
    ) -> Self {
        match master_account {
            Some(master_account) => {
                let account = master_account.create_user(account_id.expect("expected account id").to_string(), init_balance());
                storage_deposit(TOKEN_CONTRACT_ID, &master_account, SAFE_STORAGE_AMOUNT, Some(account.account_id()));
                storage_deposit(ORACLE_CONTRACT_ID, &master_account, SAFE_STORAGE_AMOUNT, Some(account.account_id()));
                near_deposit(&account, init_balance() / 2);
                Self {
                    account
                }
            },
            None => Self { account: init_simulator(None) }
        }
    }

    /*** Getters ***/
    pub fn get_token_balance(&self, account_id: Option<String>) -> u128 {
        let account_id = match account_id {
            Some(account_id) => account_id,
            None => self.account.account_id()
        };

        let res: U128 = self.account.view(
            TOKEN_CONTRACT_ID.to_string(),
            "ft_balance_of",
            json!({
                "account_id": account_id
            }).to_string().as_bytes()
        ).unwrap_json();

        res.into()
    }

    pub fn ft_transfer_call(
        &self,
        receiver: String,
        amount: u128,
        msg: String
    ) -> ExecutionResult {        
        let res = self.account.call(
            TOKEN_CONTRACT_ID.to_string(), 
            "ft_transfer_call", 
            json!({
                "receiver_id": receiver,
                "amount": U128(amount),
                "msg": msg,
                "memo": "".to_string()
            }).to_string().as_bytes(),
            DEFAULT_GAS,
            1
        );

        assert!(res.is_ok(), "ft_transfer_call failed with res: {:?}", res);
        res
    }

    pub fn dr_new(
        &self,
    ) -> ExecutionResult {
        let empty_option: Option<u8> = None;
        let transfer_res = self.account.call(
            TOKEN_CONTRACT_ID.to_string(), 
            "ft_transfer", 
            json!({
                "receiver_id": ORACLE_CONTRACT_ID,
                "amount": U128(100),
            }).to_string().as_bytes(),
            DEFAULT_GAS,
            1
        );
        assert!(transfer_res.is_ok(), "ft_transfer_call failed with res: {:?}", transfer_res);
        
        let dr_new_res = self.account.call(
            ORACLE_CONTRACT_ID.to_string(), 
            "create_data_request", 
            json!({
                "receiver_id": ORACLE_CONTRACT_ID,
                "amount": U128(100),
                "payload": json!({
                    "NewDataRequestArgs": {
                        "sources": empty_option,
                        "tags": empty_option,
                        "description": "test description",
                        "outcomes": empty_option,
                        "challenge_period": U64(1000),
                        "settlement_time": U64(10000),
                        "target_contract": TARGET_CONTRACT_ID,
                    }
                }).to_string()
            }).to_string().as_bytes(),
            DEFAULT_GAS,
            100
        );
        assert!(dr_new_res.is_ok(), "ft_transfer_call failed with res: {:?}", dr_new_res);
        dr_new_res
    }

}