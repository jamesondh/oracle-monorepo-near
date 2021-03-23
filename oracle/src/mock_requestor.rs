use crate::*;
use near_sdk::json_types::{ U64, U128 };

impl Contract {
    pub fn get_tvl_for_request(&self, _request_id: U64) -> U128 {
        // TODO: Check if requests exists
        5.into()
    }
}