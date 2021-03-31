#![allow(clippy::too_many_arguments)]

use near_sdk::{ AccountId, Balance, env, near_bindgen };
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::collections::{ Vector, LookupMap };
use near_sdk::json_types::{ ValidAccountId, U64, U128 };

mod types;
mod data_request;
mod fungible_token_receiver;
mod callback_args;
mod whitelist;
mod oracle_config;
mod storage_manager;
mod helpers;

/// Mocks
mod mock_requestor;
mod mock_token;
mod mock_target_contract;

use callback_args::*;

use types::{ Timestamp, Duration };
use data_request::{ DataRequest };

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize )]
pub struct Contract {
    pub whitelist: whitelist::Whitelist,
    pub config: oracle_config::OracleConfig,
    pub data_requests: Vector<DataRequest>,
    pub validity_bond: U128,
    pub stake_token: mock_token::Token,
    pub validity_bond_token: mock_token::Token,
    // Storage map
    pub accounts: LookupMap<AccountId, Balance>
}

impl Default for Contract {
    fn default() -> Self {
        env::panic(b"Contract should be initialized before usage")
    }
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        initial_whitelist: Option<Vec<ValidAccountId>>,
        config: oracle_config::OracleConfig
    ) -> Self {
        Self {
            whitelist: whitelist::Whitelist::new(initial_whitelist),
            config,
            data_requests: Vector::new(b"dr".to_vec()),
            validity_bond: 1.into(),
            accounts: LookupMap::new(b"a".to_vec()),

            // Mock
            stake_token: mock_token::Token::default_new(b"st".to_vec()),
            validity_bond_token: mock_token::Token::default_new(b"vbt".to_vec()),
        }
    }
}

impl Contract {
    fn assert_gov(&self) {
        assert_eq!(
            self.config.gov,
            env::predecessor_account_id(),
            "This method is only callable by the governance contract {}",
            self.config.gov
        );
    }
}