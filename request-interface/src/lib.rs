use near_sdk::{env, near_bindgen, AccountId, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

near_sdk::setup_alloc!();

mod fungible_token_handler;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct RequestInterfaceContract {
    pub oracle: AccountId,
    pub stake_token: AccountId,
    pub tvl: Balance
}

impl Default for RequestInterfaceContract {
    fn default() -> Self {
        env::panic(b"Contract should be initialized before usage")
    }
}

// Private methods
impl RequestInterfaceContract {
    pub fn assert_oracle(&self) {
        assert_eq!(&env::predecessor_account_id(), &self.oracle, "ERR_INVALID_ORACLE_ADDRESS");
    }
}

#[near_bindgen]
impl RequestInterfaceContract {
    #[init]
    pub fn new(
        oracle: AccountId,
        stake_token: AccountId
    ) -> Self {
        Self {
            oracle,
            stake_token,
            tvl: 0
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn alice() -> AccountId {
        "alice.near".to_string()
    }

    fn request_interface() -> AccountId {
        "request-interface.near".to_string()
    }

    fn token() -> AccountId {
        "token.near".to_string()
    }

    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: alice(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    #[should_panic(expected = "ERR_INVALID_ORACLE_ADDRESS")]
    fn ri_not_oracle() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let contract = RequestInterfaceContract::new(
            request_interface(),
            token()
        );
        contract.request_ft_transfer(
            100,
            alice()
        );
    }
}
