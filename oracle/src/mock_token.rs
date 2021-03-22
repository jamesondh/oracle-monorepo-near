use crate::Contract;
use near_sdk::{ AccountId, env, Balance };
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::json_types::{U128};
use near_sdk::collections::{LookupMap};

const DEFAULT_BALANCE: u128 = 10000000000000000000000000000;


/// Policy item, defining how many votes required to approve up to this much amount.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Token {
    pub balances: LookupMap<AccountId, u128>
}

impl Token {

    pub fn default_new() -> Self {
        let mut balances = LookupMap::new(b"ftb".to_vec());
        balances.insert(&env::predecessor_account_id(), &DEFAULT_BALANCE);
        Self {
            balances
        }
    }

    pub fn deposit(&mut self, receiver: AccountId, amount: u128) {
        let receiver_bal: u128 = self.get_balance_expect(receiver.to_string()).into();
        self.balances.insert(
            &receiver, 
            &(receiver_bal + amount)
        );
    }
    
    pub fn withdraw(&mut self, sender: AccountId, amount: u128) {
        let sender_bal: u128 = self.get_balance_expect(sender).into();
        assert!(sender_bal >= amount, "sender does not have enough tokens");
        self.balances.insert(
            &env::predecessor_account_id(), 
            &(sender_bal - amount)
        );
    }

    pub fn transfer(&mut self, new_owner_id: AccountId, amount: U128) {
        self.withdraw(env::predecessor_account_id(), amount.into());
        self.deposit(new_owner_id, amount.into());
    }
    
    pub fn internal_transfer(&mut self, owner_id: AccountId, new_owner_id: AccountId, amount: U128) {
        self.withdraw(owner_id, amount.into());
        self.deposit(new_owner_id, amount.into());
    }

    pub fn get_balance_expect(&self, owner_id: AccountId) -> U128 {
        self.balances.get(&owner_id)
            .unwrap_or(0)
            .into()
    }
}

pub trait FLXExternal {
    fn transfer_call(&mut self, receiver_id: String, amount: U128, msg: String);
}

impl FLXExternal for Contract {
    // Transfer call mock
    fn transfer_call(
        &mut self, 
        receiver_id: String, 
        amount: U128, 
        msg: String
    ) {
        self.token.internal_transfer(env::predecessor_account_id(), receiver_id, amount.into());
        let tokens_unspent: u128 = self.on_transfer_call(env::predecessor_account_id(), amount.into(), msg).into();
        if tokens_unspent > 0 {
            self.token.deposit(env::predecessor_account_id(), tokens_unspent.into());
        }
    }
}



#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod mock_token_basic_tests {
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    use super::*;

    const ZERO_U128: Balance = 0u128;

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
    fn invalid_account_id() -> AccountId {
        "".to_string()
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
    fn contract_creation_with_new() {
        testing_env!(get_context(carol()));
        let contract = Contract::new(1000.into());
        let carol_balance: u128 = contract.token.get_balance_expect(carol()).into();
        assert_eq!(carol_balance, DEFAULT_BALANCE);
    }

    #[test]
    fn transfer_works() {
        testing_env!(get_context(carol()));
        let mut contract = Contract::new(1000.into());
        let carol_balance: u128 = contract.token.get_balance_expect(carol()).into();
        assert_eq!(carol_balance, DEFAULT_BALANCE);
        
        let send_amount = 10000;
        contract.token.transfer(bob(), send_amount.into());
        let bob_balance: u128 = contract.token.get_balance_expect(bob()).into();
        let carol_new_balance: u128 = contract.token.get_balance_expect(carol()).into();
        
        assert_eq!(bob_balance, send_amount);
        assert_eq!(carol_new_balance, carol_balance - send_amount);
    }
    
    #[test]
    fn transfer_call_works() {
        testing_env!(get_context(carol()));
        let mut contract = Contract::new(1000.into());
        let carol_balance: u128 = contract.token.get_balance_expect(carol()).into();
        assert_eq!(carol_balance, DEFAULT_BALANCE);
        
        let send_amount = 10000;
        contract.transfer_call(bob(), send_amount.into(), "".to_string());
        let bob_balance: u128 = contract.token.get_balance_expect(bob()).into();
        let carol_new_balance: u128 = contract.token.get_balance_expect(carol()).into();
        
        assert_eq!(bob_balance, send_amount);
        assert_eq!(carol_new_balance, carol_balance - send_amount);
    }
    

    #[test]
    #[should_panic(expected = "sender does not have enough tokens")]
    fn transfer_fails_insufficient_funds() {
        testing_env!(get_context(carol()));
        let mut contract = Contract::new(1000.into());
        let carol_balance: u128 = contract.token.get_balance_expect(carol()).into();
        assert_eq!(carol_balance, DEFAULT_BALANCE);
        
        let send_amount = DEFAULT_BALANCE + 1;
        contract.token.transfer(bob(), send_amount.into());
    }

    #[test]
    #[should_panic(expected = "sender does not have enough tokens")]
    fn transfer_fails_no_funds() {
        testing_env!(get_context(carol()));
        let mut contract = Contract::new(1000.into());
        let carol_balance: u128 = contract.token.get_balance_expect(carol()).into();
        assert_eq!(carol_balance, DEFAULT_BALANCE);
        
        let send_amount = DEFAULT_BALANCE + 1;
        contract.token.transfer(bob(), send_amount.into(),);
    }

    #[test]
    #[should_panic(expected = "sender does not have enough tokens")]
    fn transfer_call_fails_insufficient_funds() {
        testing_env!(get_context(carol()));
        let mut contract = Contract::new(1000.into());
        let carol_balance: u128 = contract.token.get_balance_expect(carol()).into();
        assert_eq!(carol_balance, DEFAULT_BALANCE);
        
        let send_amount = DEFAULT_BALANCE + 1;
        contract.transfer_call(bob(), send_amount.into(), "".to_string());
    }

    #[test]
    #[should_panic(expected = "sender does not have enough tokens")]
    fn transfer_call_fails_no_funds() {
        testing_env!(get_context(carol()));
        let mut contract = Contract::new(1000.into());
        let carol_balance: u128 = contract.token.get_balance_expect(carol()).into();
        assert_eq!(carol_balance, DEFAULT_BALANCE);
        
        let send_amount = DEFAULT_BALANCE + 1;
        contract.transfer_call(bob(), send_amount.into(), "".to_string());
    }
}


