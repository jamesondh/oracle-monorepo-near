use crate::*;
use near_sdk::{Promise, PromiseResult, PromiseOrValue, ext_contract};
use near_sdk::serde_json::from_slice;
use crate::fungible_token::fungible_token_balance_of;

#[ext_contract]
pub trait RequestorContractExt {
    // fn get_tvl() -> Promise;
    fn request_ft_transfer(amount: Balance) -> Promise;
}

#[ext_contract(ext_self)]
trait SelfExt {
    fn proceed_dr_new(&mut self, sender: AccountId, amount: Balance, payload: NewDataRequestArgs);
    fn proceed_request_ft_from_requestor();
}

pub fn request_ft_from_requestor(receiver: AccountId, amount: Balance) -> Promise {
    requestor_contract_ext::request_ft_transfer(amount, &receiver, 0, 4_000_000_000_000)
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
    ) -> PromiseOrValue<Balance> {

        let requestor_tvl = fungible_token_balance_of(self.get_config().stake_token, sender.clone())
            .then(
                ext_self::proceed_dr_new(
                    sender,
                    amount,
                    payload,
                    &env::current_account_id(),
                    0,
                    4_000_000_000_000
                )
            );

        PromiseOrValue::Promise(requestor_tvl)
    }

    /**
     * @notice validates fetched TVL and creates the data request
     */
    pub fn proceed_dr_new(
        &mut self,
        sender: AccountId,
        amount: Balance,
        payload: NewDataRequestArgs
    ) -> Balance {

        // TODO: validate sender

        let tvl = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => env::panic(b"ERR_FAILED_FETCHING_TVL"),
            PromiseResult::Successful(value) => {
                match from_slice::<Balance>(&value) {
                    Ok(value) => value,
                    Err(_e) => panic!("ERR_INVALID_TVL"),
                }
            }
        };

        self.dr_new(sender.clone(), amount.into(), tvl, payload)
    }

    #[private]
    pub fn request_ft_from_requestor_callback(
        &mut self,
        reciever: AccountId,
        amount: Balance,
    ) -> PromiseOrValue<bool> {
        let result = request_ft_from_requestor(reciever, amount)
            .then(
                ext_self::proceed_request_ft_from_requestor(
                    &env::current_account_id(),
                    0,
                    4_000_000_000_000
                )
            );

        PromiseOrValue::Promise(result)
    }

    pub fn proceed_request_ft_from_requestor() -> bool {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => env::panic(b"ERR_FAILED_FETCHING_FT_FROM_REQUESTOR"),
            PromiseResult::Successful(value) => {
                match from_slice::<bool>(&value) {
                    Ok(_value) => true,
                    Err(_e) => false,
                }
            },
        }
    }

}
