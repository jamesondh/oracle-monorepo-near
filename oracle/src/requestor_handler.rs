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
    // fn proceed_request_ft_from_requestor();
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
        PromiseOrValue::Promise(
            // instead of calling get_tvl on requestor, call ft_balance_of directly on token
            fungible_token_balance_of(self.get_config().stake_token, sender.clone())
                .then(
                    ext_self::proceed_dr_new(
                        sender,
                        amount,
                        payload,
                        &env::current_account_id(),
                        0,
                        4_000_000_000_000
                    )
                )
        )
    }

    /**
     * @notice called in data_request to request a transfer from a request interface
     */
    #[private]
    pub fn request_ft_from_requestor(
        &mut self,
        requestor: AccountId,
        token: AccountId,
        receiver: AccountId,
        amount: Balance,
        callback: String
    ) {
        let promise0 = env::promise_create(
            requestor,
            callback.as_bytes(),
            json!({
                "token": token,
                "receiver": receiver,
                "amount": U128(amount),
            }).to_string().as_bytes(),
            0,
            4_000_000_000_000
        );

        let promise1 = env::promise_then(
            promise0,
            env::current_account_id(),
            b"proceed_request_ft_from_requestor",
            json!({}).to_string().as_bytes(),
            0,
            4_000_000_000_000
        );

        env::promise_return(promise1);
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
        assert_self();
        assert_prev_promise_successful();

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

    pub fn proceed_request_ft_from_requestor() -> bool {
        assert_self();
        assert_prev_promise_successful();

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
