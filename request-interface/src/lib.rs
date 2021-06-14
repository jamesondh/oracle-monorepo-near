use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{ Deserialize, Serialize };
use near_sdk::serde_json::json;
use near_sdk::json_types::{U64, U128};
use fungible_token_handler::fungible_token_transfer_call;

near_sdk::setup_alloc!();

mod fungible_token_handler;

pub type WrappedTimestamp = U64;
pub type WrappedBalance = U128;

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
pub struct Source {
    pub end_point: String,
    pub source_path: String
}

// Formatted data request args (user passes this)
// Excludes stake_multiplier and fixed_fee since those are global variables
//  and passed internally inside create_data_request()
#[derive(Serialize, Deserialize)]
pub struct NewDataRequestArgs {
    pub sources: Vec<Source>,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub outcomes: Option<Vec<String>>,
    pub challenge_period: WrappedTimestamp,
    pub settlement_time: WrappedTimestamp,
    pub target_contract: AccountId,
}

// Internal data request args with stake_multiplier and fixed_fee
// Matches NewDataRequestArgs in oracle/src/callback_args.rs
#[derive(Serialize, Deserialize)]
pub struct NewDataRequestArgsInternal {
    pub sources: Vec<Source>,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub outcomes: Option<Vec<String>>,
    pub challenge_period: WrappedTimestamp,
    pub settlement_time: WrappedTimestamp,
    pub target_contract: AccountId,
    pub stake_multiplier: Option<WrappedBalance>,
    pub fixed_fee: Option<WrappedBalance>
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct RequestInterfaceContract {
    pub oracle: AccountId,
    pub stake_token: AccountId,
    pub stake_multiplier: Option<WrappedBalance>,
    pub fixed_fee: Option<WrappedBalance>
}

impl Default for RequestInterfaceContract {
    fn default() -> Self {
        env::panic(b"Contract should be initialized before usage")
    }
}

// Private methods
impl RequestInterfaceContract {
    pub fn assert_oracle(&self) {
        assert_eq!(&env::predecessor_account_id(), &self.oracle, "ERR_INVALID_ORACLE_ADDRESS");
    }
}

#[near_bindgen]
impl RequestInterfaceContract {
    #[init]
    pub fn new(
        oracle: AccountId,
        stake_token: AccountId,
        stake_multiplier: Option<WrappedBalance>,
        fixed_fee: Option<WrappedBalance>
    ) -> Self {
        Self {
            oracle,
            stake_token,
            stake_multiplier,
            fixed_fee
        }
    }

    /**
     * @notice creates a new data request on the oracle (must be whitelisted on oracle first)
     * @returns ID of data request
     */
    pub fn create_data_request(
        &self,
        amount: WrappedBalance,
        payload: NewDataRequestArgs
    ) -> Promise {
        let payload_formatted = NewDataRequestArgsInternal {
            sources: payload.sources,
            tags: payload.tags,
            description: payload.description,
            outcomes: payload.outcomes,
            challenge_period: payload.challenge_period,
            settlement_time: payload.settlement_time,
            target_contract: payload.target_contract,
            stake_multiplier: self.stake_multiplier,
            fixed_fee: self.fixed_fee
        };
        fungible_token_transfer_call(
            self.stake_token.clone(),
            self.oracle.clone(),
            amount.into(),
            json!({"NewDataRequest": payload_formatted}).to_string() 
        )
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn alice() -> AccountId {
        "alice.near".to_string()
    }

    fn oracle() -> AccountId {
        "oracle.near".to_string()
    }

    fn token() -> AccountId {
        "token.near".to_string()
    }

    fn target() -> AccountId {
        "target.near".to_string()
    }

    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: alice(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 10000 * 10u128.pow(24),
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    #[should_panic(expected = "ERR_INVALID_ORACLE_ADDRESS")]
    fn ri_not_oracle() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let contract = RequestInterfaceContract::new(
            oracle(),
            token(),
            None,
            None
        );
        contract.request_ft_transfer(
            token(),
            100,
            alice()
        );
    }

    #[test]
    fn ri_create_dr_success() {
         let context = get_context(vec![], false);
        testing_env!(context);
        let contract = RequestInterfaceContract::new(
            oracle(),
            token(),
            None,
            None
        );

        contract.create_data_request(U128(100), NewDataRequestArgs{
            sources: Vec::new(),
            outcomes: Some(vec!["a".to_string()].to_vec()),
            challenge_period: U64(1500),
            settlement_time: U64(0),
            target_contract: target(),
            description: Some("a".to_string()),
            tags: None,
        });
    }
}
