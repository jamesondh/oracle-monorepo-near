#![allow(clippy::needless_pass_by_value)]
use near_sdk::{
    AccountId,
    json_types::{
        U64,
        U128,
        // ValidAccountId
    },
    serde_json::json,
    // serde_json
};

pub use near_sdk_sim::{
    ExecutionResult,
    deploy, 
    init_simulator, 
    to_yocto, 
    ContractAccount, 
    UserAccount, 
    DEFAULT_GAS
};
mod account_utils;
mod oracle_utils;
mod token_utils;
mod target_contract_utils;
mod request_interface_utils;
mod deposit;

// pub use account_utils::*;
extern crate oracle;
pub use oracle::*;
pub use types::*;
pub use account_utils::*;
use deposit::*;
use request_interface;
use target_contract;
use token;
use oracle::whitelist::CustomFeeStakeArgs;
use oracle::data_request::PERCENTAGE_DIVISOR;
use uint::construct_uint;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct u256(4);
}

type OracleContract = oracle::ContractContract;
type RequestInterfaceContract = request_interface::RequestInterfaceContractContract;
type TargetContract = target_contract::TargetContractContract;
type TokenContract = token::TokenContractContract;

pub const TOKEN_CONTRACT_ID: &str = "token";
pub const ORACLE_CONTRACT_ID: &str = "oracle";
pub const REQUEST_INTERFACE_CONTRACT_ID: &str = "requestor";
pub const TARGET_CONTRACT_ID: &str = "target";
pub const SAFE_STORAGE_AMOUNT: u128 = 1250000000000000000000;
pub const VALIDITY_BOND: u128 = 100;

pub fn calc_product(a: u128, b: u128, divisor: u128) -> u128 {
    let a_u256 = u256::from(a);
    let b_u256 = u256::from(b);
    let divisor_u256 = u256::from(divisor);

    (a_u256 * b_u256 / divisor_u256).as_u128()
}

pub fn calc_bond_size(validity_bond: u128, round: u32, multiplier: Option<u16>) -> u128 {
    calc_product(validity_bond * 2u128.pow(round+1), multiplier.unwrap_or(10000).into(), PERCENTAGE_DIVISOR.into())
}

// Load in contract bytes
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    ORACLE_WASM_BYTES => "../res/oracle.wasm",
    REQUEST_INTERFACE_WASM_BYTES => "../res/request_interface.wasm",
    TARGET_CONTRACT_WASM_BYTES => "../res/target_contract.wasm",
    TOKEN_WASM_BYTES => "../res/token.wasm"
}

pub struct TestUtils {
    pub master_account: TestAccount,
    pub oracle_contract: ContractAccount<OracleContract>,
    pub token_contract: ContractAccount<TokenContract>,
    pub request_interface_contract: ContractAccount<RequestInterfaceContract>,
    pub target_contract: ContractAccount<TargetContract>,
    pub alice: account_utils::TestAccount,
    pub bob: account_utils::TestAccount,
    pub carol: account_utils::TestAccount,
    pub jasper: account_utils::TestAccount,
    pub peter: account_utils::TestAccount,
    pub illia: account_utils::TestAccount,
    pub vitalik: account_utils::TestAccount,
    pub treasurer: account_utils::TestAccount,
}

pub struct TestSetupArgs {
    pub custom_fee: CustomFeeStakeArgs,
    pub validity_bond: u128,
    pub final_arbitrator_invoke_amount: u128
}

impl TestUtils {
    pub fn init(
        test_setup_args: Option<TestSetupArgs>
    ) -> Self {
        let args = test_setup_args.unwrap_or(
            TestSetupArgs {
                custom_fee: CustomFeeStakeArgs::None,
                validity_bond: VALIDITY_BOND,
                final_arbitrator_invoke_amount: 2500
            }
        );

        let master_account = TestAccount::new(None, None);
        let token_init_res = token_utils::TokenUtils::new(&master_account); // Init token
        let oracle_init_res = oracle_utils::OracleUtils::new(&master_account, args.custom_fee, args.validity_bond, args.final_arbitrator_invoke_amount);  // Init oracle
        let request_interface_init_res = request_interface_utils::RequestInterfaceUtils::new(&master_account);
        let target_contract_init_res = target_contract_utils::TargetContractUtils::new(&master_account);

        Self {
            alice: TestAccount::new(Some(&master_account.account), Some("alice")),
            bob: TestAccount::new(Some(&master_account.account), Some("bob")),
            carol: TestAccount::new(Some(&master_account.account), Some("carol")),
            jasper: TestAccount::new(Some(&master_account.account), Some("jasper")),
            peter: TestAccount::new(Some(&master_account.account), Some("peter")),
            illia: TestAccount::new(Some(&master_account.account), Some("illia")),
            vitalik: TestAccount::new(Some(&master_account.account), Some("vitalik")),
            treasurer: TestAccount::new(Some(&master_account.account), Some("treasurer")),
            master_account: master_account,
            request_interface_contract: request_interface_init_res.contract,
            target_contract: target_contract_init_res.contract,
            oracle_contract: oracle_init_res.contract,
            token_contract: token_init_res.contract
        }
    }
}

