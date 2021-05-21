use crate::*;

use near_sdk::{
    AccountId,
    Gas,
    Promise,
    json_types::U128,
    ext_contract,
};

#[ext_contract]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

const GAS_BASE_TRANSFER: Gas = 5_000_000_000_000;

pub fn fungible_token_transfer(token_account_id: AccountId, receiver_id: AccountId, value: u128) -> Promise {
    fungible_token::ft_transfer(
        receiver_id,
        U128(value),
        None,

        // Near params
        &token_account_id,
        1,
        GAS_BASE_TRANSFER
    )
}

#[near_bindgen]
impl RequestInterfaceContract {
    pub fn request_ft_transfer(
        &self,
        amount: Balance,
        receiver_id: AccountId
    ) -> Promise {
        assert_eq!(env::current_account_id(), self.oracle.clone(), "ERR_MUST_BE_ORACLE");
        fungible_token_transfer(self.stake_token.clone(), receiver_id.clone(), amount)
    }
}
