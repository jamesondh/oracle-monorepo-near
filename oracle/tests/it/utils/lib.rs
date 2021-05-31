#![allow(clippy::needless_pass_by_value)]
use std::convert::TryInto;
use near_sdk::{
    PendingContractTx,
    AccountId,
    json_types::{
        U64,
        U128,
        // ValidAccountId
    },
    serde_json::json,
    // serde_json
};

use near_sdk_sim::{
    ExecutionResult,
    deploy, 
    init_simulator, 
    to_yocto, 
    ContractAccount, 
    UserAccount, 
    STORAGE_AMOUNT,
    DEFAULT_GAS
};
mod account_utils;
mod oracle_utils;
mod token_utils;

// pub use account_utils::*;
extern crate oracle;
pub use oracle::*;
pub use account_utils::*;
use request_interface;
use target_contract;
use token;

type OracleContract = oracle::ContractContract;
type RequestInterfaceContract = request_interface::RequestInterfaceContractContract;
type TargetContract = target_contract::TargetContractContract;
type TokenContract = token::TokenContractContract;

pub const TOKEN_CONTRACT_ID: &str = "token";
pub const ORACLE_CONTRACT_ID: &str = "oracle";
pub const REQUEST_INTERFACE_CONTRACT_ID: &str = "requestor";
pub const TARGET_CONTRACT_ID: &str = "target";
pub const SAFE_STORAGE_AMOUNT: u128 = 1250000000000000000000;

// Load in contract bytes
near_sdk_sim::lazy_static! {
    static ref ORACLE_WASM_BYTES: &'static [u8] = include_bytes!("../../../../res/oracle.wasm").as_ref();
    static ref REQUEST_INTERFACE_WASM_BYTES: &'static [u8] = include_bytes!("../../../../res/request_interface.wasm").as_ref();
    static ref TARGET_CONTRACT_WASM_BYTES: &'static [u8] = include_bytes!("../../../../res/target_contract.wasm").as_ref();
    static ref TOKEN_WASM_BYTES: &'static [u8] = include_bytes!("../../../../res/token.wasm").as_ref();
}

pub struct TestUtils {
    pub master_account: TestAccount,
    pub oracle_contract: ContractAccount<OracleContract>,
    pub token_contract: ContractAccount<TokenContract>,
    // pub request_interface_contract: ContractAccount<RequestInterfaceContract>,
    // pub target_contract: ContractAccount<TargetContract>,
    pub alice: account_utils::TestAccount,
    pub bob: account_utils::TestAccount,
    pub carol: account_utils::TestAccount
}

impl TestUtils {
    pub fn init(
        gov_id: AccountId
    ) -> Self {
        let master_account = TestAccount::new(None, None);
        let token_init_res = token_utils::TokenUtils::new(&master_account); // Init token
        let oracle_init_res = oracle_utils::OracleUtils::new(&master_account);  // Init oracle
        // let request_interface_init_res = request_interface_utils::RequestInterface::new(&master_account);
        // let target_contract_init_res = target_contract_utils::TargetContract::new(&master_account);
        
        Self {
            alice: TestAccount::new(Some(&master_account.account), Some("alice")),
            bob: TestAccount::new(Some(&master_account.account), Some("bob")),
            carol: TestAccount::new(Some(&master_account.account), Some("carol")),
            master_account: master_account,
            // request_interface_contract: request_interface_init_res.contract,
            // target_contract: target_contract_init_res.contract,
            oracle_contract: oracle_init_res.contract,
            token_contract: token_init_res.contract, // should be doable like oracle and amm
        }
    }
}

