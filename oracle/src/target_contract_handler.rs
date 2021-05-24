use crate::*;
use crate::data_request::Outcome;
use near_sdk::{Promise, PromiseOrValue, ext_contract};
use near_sdk::json_types::U64;

#[ext_contract]
pub trait TargetContractExt {
    fn set_outcome(request_id: U64, outcome: Outcome) -> Promise;
}

pub fn ext_set_outcome(request_id: U64, outcome: Outcome, target: AccountId) -> Promise {
    target_contract_ext::set_outcome(request_id, outcome, &target, 0, 4_000_000_000_000)
}

#[near_bindgen]
impl Contract {

    #[private]
    pub fn tc_set_outcome(&self, request_id: U64, outcome: Outcome, target: AccountId) -> PromiseOrValue<bool> {
        PromiseOrValue::Promise(ext_set_outcome(request_id, outcome, target))
    }

}
