use crate::*;
use near_sdk::json_types::{ U64 };
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::serde::{ Deserialize, Serialize };
use near_sdk::{ AccountId, Gas, ext_contract, Promise };
use data_request::Outcome;

#[ext_contract]
pub trait TargetContractExtern {
    fn set_outcome(request_id: U64, requestor: AccountId, outcome: Outcome, tags: Option<Vec<String>>, final_arbitrator_triggered: bool);
    fn claim_fee(request_id: U64, fee_percentage: u16);
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub struct TargetContract(pub AccountId);

const GAS_BASE_SET_OUTCOME: Gas = 250_000_000_000_000;

impl TargetContract {
    pub fn claim_fee(
        &self,
        request_id: U64,
        fee_percentage: u16
    ) -> Promise {
        target_contract_extern::claim_fee(
            request_id, 
            fee_percentage,

            // NEAR params
            &self.0,
            1,
            GAS_BASE_SET_OUTCOME,
        )
    }

    pub fn set_outcome(
        &self,
        request_id: U64,
        requestor: AccountId,
        outcome: data_request::Outcome,
        tags: Option<Vec<String>>,
        final_arbitrator_triggered: bool
    ) -> Promise {
        target_contract_extern::set_outcome(
            request_id,
            requestor,
            outcome,
            tags,
            final_arbitrator_triggered,

            // NEAR params
            &self.0,
            1, 
            GAS_BASE_SET_OUTCOME / 10,
        )
    }
}
