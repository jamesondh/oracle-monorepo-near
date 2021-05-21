use crate::*;
use near_sdk::{Promise, PromiseResult, PromiseOrValue, ext_contract};
use near_sdk::serde_json::from_slice;

#[ext_contract]
pub trait RequestorContractExt {
    fn get_tvl() -> Promise;
}

#[ext_contract(ext_self)]
trait SelfExt {
    fn proceed_dr_new(&mut self, sender: AccountId, amount: Balance, payload: NewDataRequestArgs);
}

pub fn fetch_tvl(requestor: AccountId) -> Promise {
    requestor_contract_ext::get_tvl(&requestor, 0, 4_000_000_000_000)
}

#[near_bindgen]
impl Contract {

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

        let requestor_tvl = fetch_tvl(sender.clone())
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

}
