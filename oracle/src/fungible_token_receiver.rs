use crate::*;
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
        let payload: Payload =  serde_json::from_str(&msg).expect("Failed to parse the payload, invalid `msg` format");
        match payload {
            Payload::NewDataRequest(payload) => self.dr_new(sender, amount.into(), payload),
            Payload::StakeDataRequest(payload) => self.dr_stake(sender, amount.into(), payload),
        }.into()
    }
}