use near_sdk::{env, near_bindgen, AccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{ Deserialize, Serialize };

near_sdk::setup_alloc!();

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Outcome {
    Answer(String),
    Invalid
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct TargetContract {
    pub oracle: AccountId,
    pub outcome: Option<Outcome>
}

impl Default for TargetContract {
    fn default() -> Self {
        env::panic(b"Contract should be initialized before usage")
    }
}

// Private methods
impl TargetContract {
    pub fn assert_oracle(&self) {
        assert_eq!(&env::predecessor_account_id(), &self.oracle, "ERR_INVALID_ORACLE_ADDRESS");
    }
}

#[near_bindgen]
impl TargetContract {
    #[init]
    pub fn new(
        oracle: AccountId
    ) -> Self {
        Self {
            oracle,
            outcome: None
        }
    }

    pub fn set_outcome(
        &mut self,
        outcome: Outcome
    ) {
        self.assert_oracle();
        self.outcome = Some(outcome);
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

    fn get_context(predecessor_account_id: AccountId) -> VMContext {
        VMContext {
            current_account_id: alice(),
            signer_account_id: alice(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    fn tc_outcome_initialized() {
        let context = get_context(alice());
        testing_env!(context);
        let contract = TargetContract::new(
            oracle()
        );
        assert_eq!(contract.outcome, None);
    }

    #[test]
    #[should_panic(expected = "ERR_INVALID_ORACLE_ADDRESS")]
    fn tc_set_outcome_not_oracle() {
        let context = get_context(alice());
        testing_env!(context);
        let mut contract = TargetContract::new(
            oracle()
        );
        contract.set_outcome(Outcome::Answer("outcome".to_string()));
    }

    #[test]
    fn tc_set_outcome_success() {
        let context = get_context(oracle());
        testing_env!(context);
        let mut contract = TargetContract::new(
            oracle()
        );
        contract.set_outcome(Outcome::Answer("outcome".to_string()));
    }
}
