use crate::Contract;
use std::convert::TryInto;

use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::serde::{ Serialize, Deserialize };
use near_sdk::AccountId;
use near_sdk::collections::LookupSet;
use near_sdk::json_types::ValidAccountId;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RegistryEntry {
    pub interface_name: String,
    pub contract_entry: AccountId,
    pub callback: String,
    pub tvs_method: String,
    pub rvs_method: String,
    pub code_base_url: String
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Whitelist(LookupSet<AccountId>);

// TODO: Add optional initial requestor(s)
impl Whitelist {
    pub fn new(initial_whitelist: Option<Vec<ValidAccountId>>) -> Self {
        let mut whitelist: LookupSet<AccountId> = LookupSet::new(b"wlr".to_vec());

        if initial_whitelist.is_some() {
            for valid_account_id in initial_whitelist.unwrap() {
                let account_id: AccountId = valid_account_id.try_into().expect("Invalid account_id");
                whitelist.insert(&account_id);
            }
        }

        Self(whitelist)
    }

    pub fn contains(&self, requestor: AccountId) -> bool {
        self.0.contains(&requestor)
    }
}

trait WhitelistHandler {
    fn add_to_whitelist(&mut self, new_requestor: ValidAccountId);
    fn whitelist_contains(&self, requestor: AccountId) -> bool;
}

impl WhitelistHandler for Contract {
    fn add_to_whitelist(&mut self, new_requestor: ValidAccountId) {
        // TODO: Assert governance predecessor
        let new_requestor = new_requestor.try_into().expect("Invalid account id");
        self.whitelist.0.insert(&new_requestor);
    }
   
    fn whitelist_contains(&self, requestor: AccountId) -> bool {
        self.whitelist.contains(requestor)
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

    fn carol() -> AccountId {
        "carol.near".to_string()
    }

    fn token() -> AccountId {
        "token.near".to_string()
    }
 
    fn _gov() -> AccountId {
        "gov.near".to_string()
    }

    fn to_valid(account: AccountId) -> ValidAccountId {
        account.try_into().expect("invalid account")
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
    fn setting_initial_whitelist() {
        testing_env!(get_context(carol()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let contract = Contract::new(whitelist);
        let alice_is_whitelisted = contract.whitelist.contains(alice());
        let bob_is_whitelisted = contract.whitelist.contains(bob());
        let carol_is_whitelisted = contract.whitelist.contains(carol());
        assert!(!alice_is_whitelisted);
        assert!(bob_is_whitelisted);
        assert!(carol_is_whitelisted);
    }

    // TODO: add gov assertion tests
}


