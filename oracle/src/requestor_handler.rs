use crate::*;
use crate::helpers::{assert_self, assert_prev_promise_successful};
use near_sdk::{PromiseResult, PromiseOrValue, ext_contract};
use near_sdk::serde_json::{from_slice, json};
use crate::fungible_token::fungible_token_balance_of;

// #[ext_contract]
// pub trait RequestorContractExt {
//     // fn get_tvl() -> Promise;
//     fn request_ft_transfer(token: AccountId, amount: Balance) -> Promise;
// }

#[ext_contract(ext_self)]
trait SelfExt {
    fn proceed_dr_new(&mut self, sender: AccountId, amount: Balance, payload: NewDataRequestArgs);
}

#[near_bindgen]
impl Contract {

    /**
     * @notice called in ft_on_transfer to chain together fetching of TVL and data request creation
     */
    #[private]
    pub fn ft_dr_new_callback(
        &mut self,
        sender: AccountId,
        amount: Balance,
        payload: NewDataRequestArgs
    ) -> u128 {
        self.dr_new(sender.clone(), amount.into(), 0, payload)
    }
}