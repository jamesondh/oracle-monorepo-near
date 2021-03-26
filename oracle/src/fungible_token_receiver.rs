use crate::*;
use storage_manager::{ STORAGE_PRICE_PER_BYTE };
use near_sdk::serde::{ Serialize, Deserialize };
use near_sdk::serde_json;

#[derive(Serialize, Deserialize)]
pub enum Payload {
    NewDataRequest(NewDataRequestArgs),
    StakeDataRequest(StakeDataRequestArgs)
}

pub trait FungibleTokenReceiver { 
    // @returns amount of unused tokens
    fn ft_on_transfer(&mut self, sender: AccountId, amount: U128, msg: String) -> U128;
}

impl FungibleTokenReceiver for Contract {
    // @returns amount of unused tokens
    fn ft_on_transfer(
        &mut self,
        sender: AccountId,
        amount: U128,
        msg: String
    ) -> U128 {
        let initial_storage_usage = env::storage_usage();
        let initial_user_balance = self.accounts.get(&env::predecessor_account_id()).unwrap_or(0);
        let payload: Payload =  serde_json::from_str(&msg).expect("Failed to parse the payload, invalid `msg` format");
        let unspent: U128 = match payload {
            Payload::NewDataRequest(payload) => self.dr_new(sender, amount.into(), payload),
            Payload::StakeDataRequest(payload) => self.dr_stake(sender, amount.into(), payload),
        }.into();

        if env::storage_usage() >= initial_storage_usage {
            // used more storage, deduct from balance
            let difference : u128 = u128::from(env::storage_usage() - initial_storage_usage);
            self.accounts.insert(&env::predecessor_account_id(), &(initial_user_balance - difference * STORAGE_PRICE_PER_BYTE));
        } else {
            // freed up storage, add to balance
            let difference : u128 = u128::from(initial_storage_usage - env::storage_usage());
            self.accounts.insert(&env::predecessor_account_id(), &(initial_user_balance + difference * STORAGE_PRICE_PER_BYTE));
        }

        unspent
    }
}