use crate::*;
use std::convert::TryInto;

use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::json_types::{ U64 };
use near_sdk::serde::{ Deserialize, Serialize };
use near_sdk::{ env, Balance, AccountId };
use near_sdk::collections::{ Vector, LookupMap };

use crate::types::{ Timestamp, Duration, WrappedBalance };

const PERCENTAGE_DIVISOR: u16 = 10_000;

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Outcome {
    Answer(String),
    Invalid
}

pub enum WindowStakeResult {
    Incorrect(Balance), // Round bonded outcome was correct
    Correct(CorrectStake), // Round bonded outcome was incorrect
    NoResult // Last / non-bonded window
}

pub struct CorrectStake {
    pub bonded_stake: Balance,
    pub user_stake: Balance,
}

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
pub struct Source {
    pub end_point: String,
    pub source_path: String
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ResolutionWindow {
    pub dr_id: u64,
    pub round: u16,
    pub end_time: Timestamp,
    pub bond_size: Balance,
    pub outcome_to_stake: LookupMap<Outcome, Balance>,
    pub user_to_outcome_to_stake: LookupMap<AccountId, LookupMap<Outcome, Balance>>,
    pub bonded_outcome: Option<Outcome>,
}

trait ResolutionWindowChange {
    fn new(dr_id: u64, round: u16, prev_bond: Balance, challenge_period: u64) -> Self;
    fn stake(&mut self, sender: AccountId, outcome: Outcome, amount: Balance) -> Balance;
    fn unstake(&mut self, sender: AccountId, outcome: Outcome, amount: Balance) -> Balance;
    fn claim_for(&mut self, account_id: AccountId, final_outcome: &Outcome) -> WindowStakeResult;
}

impl ResolutionWindowChange for ResolutionWindow {
    fn new(dr_id: u64, round: u16, prev_bond: Balance, challenge_period: u64) -> Self {
        Self {
            dr_id,
            round,
            end_time: env::block_timestamp() + challenge_period,
            bond_size: prev_bond * 2,
            outcome_to_stake: LookupMap::new(format!("ots{}:{}", dr_id, round).as_bytes().to_vec()),
            user_to_outcome_to_stake: LookupMap::new(format!("utots{}:{}", dr_id, round).as_bytes().to_vec()),
            bonded_outcome: None
        }
    }

    // @returns amount to refund users because it was not staked
    fn stake(&mut self, sender: AccountId, outcome: Outcome, amount: Balance) -> Balance {
        let stake_on_outcome = self.outcome_to_stake.get(&outcome).unwrap_or(0);
        let mut user_to_outcomes = self.user_to_outcome_to_stake
            .get(&sender)
            .unwrap_or(LookupMap::new(format!("utots:{}:{}:{}", self.dr_id, self.round, sender).as_bytes().to_vec()));
        let user_stake_on_outcome = user_to_outcomes.get(&outcome).unwrap_or(0);

        let stake_open = self.bond_size - stake_on_outcome;
        let unspent = if amount > stake_open {
            amount - stake_open
        } else {
            0
        };

        let staked = amount - unspent;

        let new_stake_on_outcome = stake_on_outcome + staked;
        self.outcome_to_stake.insert(&outcome, &new_stake_on_outcome);

        let new_user_stake_on_outcome = user_stake_on_outcome + staked;
        user_to_outcomes.insert(&outcome, &new_user_stake_on_outcome);
        self.user_to_outcome_to_stake.insert(&sender, &user_to_outcomes);

        // If this stake fills the bond set final outcome which will trigger a new resolution_window to be created
        if new_stake_on_outcome == self.bond_size {
            self.bonded_outcome = Some(outcome);
        }

        unspent
    }

    // @returns amount to refund users because it was not staked
    fn unstake(&mut self, sender: AccountId, outcome: Outcome, amount: Balance) -> Balance {
        assert!(self.bonded_outcome.is_none() || self.bonded_outcome.as_ref().unwrap() != &outcome, "Cannot withdraw from bonded outcome");
        let mut user_to_outcomes = self.user_to_outcome_to_stake
            .get(&sender)
            .unwrap_or(LookupMap::new(format!("utots:{}:{}:{}", self.dr_id, self.round, sender).as_bytes().to_vec()));
        let user_stake_on_outcome = user_to_outcomes.get(&outcome).unwrap_or(0);
        assert!(user_stake_on_outcome >= amount, "{} has less staked on this outcome ({}) than unstake amount", sender, user_stake_on_outcome);

        let stake_on_outcome = self.outcome_to_stake.get(&outcome).unwrap_or(0);

        let new_stake_on_outcome = stake_on_outcome - amount;
        self.outcome_to_stake.insert(&outcome, &new_stake_on_outcome);

        let new_user_stake_on_outcome = user_stake_on_outcome - amount;
        user_to_outcomes.insert(&outcome, &new_user_stake_on_outcome);
        self.user_to_outcome_to_stake.insert(&sender, &user_to_outcomes);

        amount
    }

    fn claim_for(&mut self, account_id: AccountId, final_outcome: &Outcome) -> WindowStakeResult {
        // Check if there is a bonded outcome, if there is none it means it can be ignored in payout calc since it can only be the final unsuccessful window
        match &self.bonded_outcome {
            Some(bonded_outcome) => {
                // If the bonded outcome for this window is equal to the finalized outcome the user's stake in this window and the total amount staked should be returned (which == `self.bond_size`)
                if bonded_outcome == final_outcome {
                    WindowStakeResult::Correct(CorrectStake {
                        bonded_stake: self.bond_size,
                        // Get the users stake in this outcome for this window
                        user_stake:  match &mut self.user_to_outcome_to_stake.get(&account_id) {
                            Some(outcome_to_stake) => {
                                outcome_to_stake.remove(&bonded_outcome).unwrap_or(0)
                            },
                            None => 0
                        }
                    })
                // Else if the bonded outcome for this window is not equal to the finalized outcome the user's stake in this window only the total amount that was staked on the incorrect outcome should be returned
                } else {
                    WindowStakeResult::Incorrect(self.bond_size)
                }
            },
            None => WindowStakeResult::NoResult // Return `NoResult` for non-bonded window
        }
    }

}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct DataRequest {
    pub id: u64,
    pub sources: Vec<Source>,
    pub settlement_time: Timestamp,
    pub outcomes: Option<Vec<String>>,
    pub requestor: mock_requestor::Requestor,
    pub finalized_outcome: Option<Outcome>,
    pub resolution_windows: Vector<ResolutionWindow>,
    pub config: oracle_config::OracleConfig, // Config at initiation
    pub initial_challenge_period: Duration,
    pub final_arbitrator_triggered: bool,
    pub target_contract: mock_target_contract::TargetContract

}

trait DataRequestChange {
    fn new(sender: AccountId, id: u64, config: oracle_config::OracleConfig, request_data: NewDataRequestArgs) -> Self;
    fn stake(&mut self, sender: AccountId, outcome: Outcome, amount: Balance) -> Balance;
    fn unstake(&mut self, sender: AccountId, round: u16, outcome: Outcome, amount: Balance) -> Balance;
    fn finalize(&mut self);
    fn invoke_final_arbitrator(&mut self, bond_size: Balance) -> bool;
    fn finalize_final_arbitrator(&mut self, outcome: Outcome);
    fn claim(&mut self, account_id: String) -> Balance;
    fn return_validity_bond(&self, token: &mut mock_token::Token);
}

impl DataRequestChange for DataRequest {
    fn new(
        sender: AccountId,
        id: u64,
        config: oracle_config::OracleConfig,
        request_data: NewDataRequestArgs
    ) -> Self {
        let resolution_windows = Vector::new(format!("rw{}", id).as_bytes().to_vec());
        Self {
            id: id,
            sources: request_data.sources,
            settlement_time: request_data.settlement_time,
            outcomes: request_data.outcomes,
            requestor: mock_requestor::Requestor(sender),
            finalized_outcome: None,
            resolution_windows,
            config,
            initial_challenge_period: request_data.challenge_period,
            final_arbitrator_triggered: false,
            target_contract: mock_target_contract::TargetContract(request_data.target_contract)
        }
    }

    // @returns amount of tokens that didn't get staked
    fn stake(&mut self, sender: AccountId, outcome: Outcome, amount: Balance) -> Balance {
        let mut window = self.resolution_windows
            .iter()
            .last()
            .unwrap_or(
                ResolutionWindow::new(self.id, 0, self.calc_resolution_bond(), self.initial_challenge_period)
            );

        let unspent = window.stake(sender, outcome, amount);

        // If first window push it to vec, else replace updated window struct
        if self.resolution_windows.len() == 0 {
            self.resolution_windows.push(&window);
        } else {
            self.resolution_windows.replace(
                self.resolution_windows.len() - 1, // Last window
                &window
            );
        }
        // Check if this stake is bonded for the current window and if the final arbitrator should be invoked.
        // If the final arbitrator is invoked other stake won't come through.
        if window.bonded_outcome.is_some() && !self.invoke_final_arbitrator(window.bond_size) {
            self.resolution_windows.push(
                &ResolutionWindow::new(
                    self.id,
                    self.resolution_windows.len() as u16,
                    window.bond_size,
                    self.config.default_challenge_window_duration
                )
            );

        }

        unspent
    }

    // @returns amount of tokens that didn't get staked
    fn unstake(&mut self, sender: AccountId, round: u16, outcome: Outcome, amount: Balance) -> Balance {
        let mut window = self.resolution_windows
            .get(round as u64)
            .unwrap_or(
                ResolutionWindow::new(self.id, 0, self.calc_resolution_bond(), self.initial_challenge_period)
            );

        window.unstake(sender, outcome, amount)
    }

    fn finalize(&mut self) {
        self.finalized_outcome = self.get_final_outcome();
    }

    // @returns wether final arbitrator was triggered
    fn invoke_final_arbitrator(&mut self, bond_size: Balance) -> bool {
        let should_invoke = bond_size >= self.config.final_arbitrator_invoke_amount;
        if should_invoke { self.final_arbitrator_triggered = true }
        self.final_arbitrator_triggered
    }

    fn finalize_final_arbitrator(&mut self, outcome: Outcome) {
        self.finalized_outcome = Some(outcome);
    }

    fn claim(&mut self, account_id: String) -> Balance {
        // Metrics for calculating payout
        let mut total_correct_staked = 0;
        let mut total_incorrect_staked = 0;
        let mut user_correct_stake = 0;

        // Metrics we need to calculate resolution round result
        let resolution_payout = self.calc_resolution_fee_payout();
        let mut resolution_round_earnings = 0;

        // For any round after the resolution round handle generically
        for round in 0..self.resolution_windows.len() {
            let mut window = self.resolution_windows.get(round).unwrap();
            let stake_state: WindowStakeResult = window.claim_for(account_id.to_string(), self.finalized_outcome.as_ref().unwrap());

            match stake_state {
                WindowStakeResult::Correct(correctly_staked) => {
                    // If it's the first round and the round was correct this should count towards the users resolution payout, it is not seen as total stake
                    if round == 0 {
                        resolution_round_earnings = helpers::calc_product(correctly_staked.user_stake, resolution_payout, correctly_staked.bonded_stake)
                    } else {
                        total_correct_staked += correctly_staked.bonded_stake;
                        user_correct_stake += correctly_staked.user_stake;
                    }
                },
                WindowStakeResult::Incorrect(incorrectly_staked) => {
                    total_incorrect_staked += incorrectly_staked
                },
                WindowStakeResult::NoResult => ()
            }

            self.resolution_windows.replace(round as u64, &window);
        };

        resolution_round_earnings + helpers::calc_product(user_correct_stake, total_incorrect_staked, total_correct_staked)
    }

    // @notice Return what's left of validity_bond to requestor
    fn return_validity_bond(&self, token: &mut mock_token::Token) {
        let bond_to_return = self.calc_validity_bond_to_return();
        if bond_to_return > 0 {
            token.transfer(
                self.requestor.0.to_string(),
                bond_to_return.into()
            );
        }
    }
}

trait DataRequestView {
    fn assert_valid_outcome(&self, outcome: &Outcome);
    fn assert_can_stake_on_outcome(&self, outcome: &Outcome);
    fn assert_not_finalized(&self);
    fn assert_finalized(&self);
    fn assert_settlement_time_passed(&self);
    fn assert_can_finalize(&self);
    fn assert_final_arbitrator(&self);
    fn assert_final_arbitrator_invoked(&self);
    fn get_final_outcome(&self) -> Option<Outcome>;
    fn get_tvl(&self) -> Balance;
    fn calc_fee(&self) -> Balance;
    fn calc_resolution_bond(&self) -> Balance;
    fn calc_validity_bond_to_return(&self) -> Balance;
    fn calc_resolution_fee_payout(&self) -> Balance;
}

impl DataRequestView for DataRequest {
    fn assert_valid_outcome(&self, outcome: &Outcome) {
        match &self.outcomes {
            Some(outcomes) => match outcome {
                Outcome::Answer(outcome) => assert!(outcomes.contains(&outcome), "Incompatible outcome"),
                Outcome::Invalid => ()
            },
            None => ()
        }
    }

    fn assert_can_stake_on_outcome(&self, outcome: &Outcome) {
        if self.resolution_windows.len() > 1 {
            let last_window = self.resolution_windows.get(self.resolution_windows.len() - 2).unwrap();
            // TODO, currently checking references are equal. In my experience checking values is safer.
            assert_ne!(&last_window.bonded_outcome.unwrap(), outcome, "Outcome is incompatible for this round");
        }
    }

    fn assert_not_finalized(&self) {
        assert!(self.finalized_outcome.is_none(), "Can't stake in finalized DataRequest");
    }

    fn assert_finalized(&self) {
        assert!(self.finalized_outcome.is_some(), "DataRequest is not finalized");
    }

    fn assert_settlement_time_passed(&self) {
        assert!(env::block_timestamp() >= self.settlement_time, "DataRequest cannot be processed yet");
    }

    fn assert_can_finalize(&self) {
        assert!(!self.final_arbitrator_triggered, "Can only be finalized by final arbitrator: {}", self.config.final_arbitrator);
        let last_window = self.resolution_windows.iter().last().expect("No resolution windows found, DataRequest not processed");
        self.assert_not_finalized();
        assert!(env::block_timestamp() >= last_window.end_time, "Challenge period not ended");
    }

    fn assert_final_arbitrator(&self) {
        assert_eq!(
            self.config.final_arbitrator,
            env::predecessor_account_id(),
            "sender is not the final arbitrator of this `DataRequest`, the final arbitrator is: {}",
            env::predecessor_account_id()
        );
    }

    fn assert_final_arbitrator_invoked(&self) {
        assert!(
            self.final_arbitrator_triggered,
            "Final arbitrator can not finalize `DataRequest` with id: {}",
            self.id
        );
    }

    fn get_final_outcome(&self) -> Option<Outcome> {
        let last_bonded_window_i = self.resolution_windows.len() - 2; // Last window after end_time never has a bonded outcome
        let last_bonded_window = self.resolution_windows.get(last_bonded_window_i).unwrap();
        last_bonded_window.bonded_outcome
    }

    fn get_tvl(&self) -> Balance {
        self.requestor.get_tvl(self.id.into()).into()
    }

    fn calc_fee(&self) -> Balance {
        let tvl = self.get_tvl();
        self.config.resolution_fee_percentage as Balance * tvl / PERCENTAGE_DIVISOR as Balance
    }

    /**
     * @notice Calculates the size of the resolution bond. If the accumulated fee is smaller than the validity bond, we payout the validity bond to validators, thus they have to stake double in order to be
     * eligible for the reward, in the case that the fee is greater than the validity bond validators need to have a cumulative stake of double the fee amount
     * @returns The size of the initial `resolution_bond` denominated in `stake_token`
     */
    fn calc_resolution_bond(&self) -> Balance {
        let fee = self.calc_fee();

        if fee > self.config.validity_bond {
            fee
        } else {
            self.config.validity_bond
        }
    }

     /**
     * @notice Calculates, how much of, the `validity_bond` should be returned to the creator, if the fees accrued are less than the validity bond only return the fees accrued to the creator
     * the rest of the bond will be paid out to resolvers. If the `DataRequest` is invalid the fees and the `validity_bond` are paid out to resolvers, the creator gets slashed.
     * @returns How much of the `validity_bond` should be returned to the creator after resolution denominated in `stake_token`
     */
    fn calc_validity_bond_to_return(&self) -> Balance {
        let fee = self.calc_fee();
        let outcome = self.finalized_outcome.as_ref().unwrap();

        match outcome {
            Outcome::Answer(_) => {
                if fee > self.config.validity_bond {
                    self.config.validity_bond
                } else {
                    fee
                }
            },
            Outcome::Invalid => 0
        }
    }

    /**
     * @notice Calculates the size of the resolution bond. If the accumulated fee is smaller than the validity bond, we payout the validity bond to validators, thus they have to stake double in order to be
     * eligible for the reward, in the case that the fee is greater than the validity bond validators need to have a cumulative stake of double the fee amount
     * @returns The size of the resolution fee paid out to resolvers denominated in `stake_token`
     */
    fn calc_resolution_fee_payout(&self) -> Balance {
        let fee = self.calc_fee();
        let outcome = self.finalized_outcome.as_ref().unwrap();

        match outcome {
            Outcome::Answer(_) => {
                if fee > self.config.validity_bond {
                    fee
                } else {
                    self.config.validity_bond
                }
            },
            Outcome::Invalid => fee + self.config.validity_bond
        }
    }
}

#[near_bindgen]
impl Contract {
    // Merge config and payload
    pub fn dr_new(&mut self, sender: AccountId, amount: Balance, payload: NewDataRequestArgs) -> Balance {
        self.assert_whitelisted(sender.to_string());
        self.assert_bond_token();
        self.dr_validate(&payload);
        assert!(amount >= self.config.validity_bond, "Validity bond not reached");

        let dr = DataRequest::new(
            sender,
            self.data_requests.len() as u64,
            self.config.clone(), // TODO: should probably trim down once we know what attributes we need stored for `DataRequest`s
            payload
        );
        self.data_requests.push(&dr);

        if amount > self.config.validity_bond {
            amount - self.config.validity_bond
        } else {
            0
        }
    }

    #[payable]
    pub fn dr_stake(&mut self, sender: AccountId, amount: Balance, payload: StakeDataRequestArgs) -> Balance {
        self.assert_stake_token();
        let mut dr = self.dr_get_expect(payload.id.into());
        dr.assert_can_stake_on_outcome(&payload.outcome);
        dr.assert_valid_outcome(&payload.outcome);
        dr.assert_not_finalized();
        dr.assert_settlement_time_passed();
        // TODO remove variable assignment?
        let _tvl = dr.get_tvl();

        let unspent_stake = dr.stake(sender, payload.outcome, amount);

        self.data_requests.replace(payload.id.into(), &dr);

        unspent_stake
    }

    #[payable]
    pub fn dr_unstake(&mut self, request_id: U64, resolution_round: u16, outcome: Outcome, amount: Balance) -> WrappedBalance {
        let initial_storage = env::storage_usage();

        let mut dr = self.dr_get_expect(request_id.into());
        let unstaked = dr.unstake(env::predecessor_account_id(), resolution_round, outcome, amount);

        helpers::refund_storage(initial_storage, env::predecessor_account_id());

        unstaked.into()
    }

    /**
     * @returns amount of tokens claimed
     */
    #[payable]
    pub fn dr_claim(&mut self, account_id: String, request_id: U64) -> Balance {
        let initial_storage = env::storage_usage();

        let mut dr = self.dr_get_expect(request_id.into());
        dr.assert_finalized();
        let payout = dr.claim(account_id.to_string());
        self.stake_token.transfer(account_id, payout.into());

        helpers::refund_storage(initial_storage, env::predecessor_account_id());

        payout
    }

    #[payable]
    pub fn dr_finalize(&mut self, request_id: U64) {
        let initial_storage = env::storage_usage();
        let mut dr = self.dr_get_expect(request_id);
        dr.assert_can_finalize();
        dr.finalize();
        self.data_requests.replace(request_id.into(), &dr);

        dr.target_contract.set_outcome(request_id, dr.finalized_outcome.as_ref().unwrap().clone());
        dr.return_validity_bond(&mut self.validity_bond_token);

        helpers::refund_storage(initial_storage, env::predecessor_account_id());
    }

    #[payable]
    pub fn dr_final_arbitrator_finalize(&mut self, request_id: U64, outcome: Outcome) {
        let initial_storage = env::storage_usage();

        let mut dr = self.dr_get_expect(request_id);
        dr.assert_final_arbitrator();
        dr.assert_valid_outcome(&outcome);
        dr.assert_final_arbitrator_invoked();
        dr.finalize_final_arbitrator(outcome.clone());

        dr.target_contract.set_outcome(request_id, outcome);
        dr.return_validity_bond(&mut self.validity_bond_token);

        helpers::refund_storage(initial_storage, env::predecessor_account_id());
    }
}

#[near_bindgen]
impl Contract {
    fn dr_get_expect(&self, id: U64) -> DataRequest {
        self.data_requests.get(id.into()).expect("DataRequest with this id does not exist")
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod mock_token_basic_tests {
    use near_sdk::{ MockedBlockchain };
    use near_sdk::{ testing_env, VMContext };
    use super::*;

    fn alice() -> AccountId {
        "alice.near".to_string()
    }

    fn bob() -> AccountId {
        "bob.near".to_string()
    }

    fn carol() -> AccountId {
        "carol.near".to_string()
    }

    fn token() -> AccountId {
        "token.near".to_string()
    }

    fn target() -> AccountId {
        "target.near".to_string()
    }

    fn gov() -> AccountId {
        "gov.near".to_string()
    }

    fn to_valid(account: AccountId) -> ValidAccountId {
        account.try_into().expect("invalid account")
    }

    fn config() -> oracle_config::OracleConfig {
        oracle_config::OracleConfig {
            gov: gov(),
            final_arbitrator: alice(),
            bond_token: token(),
            stake_token: token(),
            validity_bond: 100,
            max_outcomes: 8,
            default_challenge_window_duration: 1000,
            min_initial_challenge_window_duration: 1000,
            final_arbitrator_invoke_amount: 250,
            resolution_fee_percentage: 0,
        }
    }

    fn get_context(predecessor_account_id: AccountId) -> VMContext {
        VMContext {
            current_account_id: token(),
            signer_account_id: bob(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 1000 * 10u128.pow(24),
            account_locked_balance: 0,
            storage_usage: 10u64.pow(6),
            attached_deposit: 1000 * 10u128.pow(24),
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    #[should_panic(expected = "Invalid outcome list either exceeds min of: 2 or max of 8")]
    fn dr_new_single_outcome() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());

        contract.dr_new(bob(), 100, NewDataRequestArgs{
            sources: Vec::new(),
            outcomes: Some(vec!["a".to_string()].to_vec()),
            settlement_time: 0,
            challenge_period: 1500,
            target_contract: target(),
        });
    }


    #[test]
    #[should_panic(expected = "Err predecessor is not whitelisted")]
    fn dr_new_non_whitelisted() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());
        contract.dr_new(alice(), 100, NewDataRequestArgs{
            sources: Vec::new(),
            outcomes: None,
            settlement_time: 0,
            challenge_period: 0,
            target_contract: target(),
        });
    }

    #[test]
    #[should_panic(expected = "Only the bond token contract can call this function")]
    fn dr_new_non_bond_token() {
        testing_env!(get_context(alice()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());
        contract.dr_new(bob(), 100, NewDataRequestArgs{
            sources: Vec::new(),
            outcomes: None,
            settlement_time: 0,
            challenge_period: 0,
            target_contract: target(),
        });
    }

    #[test]
    fn dr_new_arg_source_exceed() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());

        contract.dr_new(bob(), 100, NewDataRequestArgs{
            sources: Vec::new(), // todo call with 9 sources
            outcomes: None,
            settlement_time: 0,
            challenge_period: 1000,
            target_contract: target(),
        });
    }

    #[test]
    fn dr_new_arg_outcome_exceed() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());

        contract.dr_new(bob(), 100, NewDataRequestArgs{
            sources: Vec::new(),
            outcomes: None,  // todo call with 9 outcomes
            settlement_time: 0,
            challenge_period: 1000,
            target_contract: target(),
        });
    }

    #[test]
    #[should_panic(expected = "Challenge shorter than minimum challenge period")]
    fn dr_new_arg_challenge_period_below_min() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());

        contract.dr_new(bob(), 100, NewDataRequestArgs{
            sources: Vec::new(),
            outcomes: None,
            settlement_time: 0,
            challenge_period: 999,
            target_contract: target(),
        });
    }

    #[test]
    #[should_panic(expected = "Challenge period exceeds maximum challenge period")]
    fn dr_new_arg_challenge_period_exceed() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());

        contract.dr_new(bob(), 100, NewDataRequestArgs{
            sources: Vec::new(),
            outcomes: None,
            settlement_time: 0,
            challenge_period: 3001,
            target_contract: target(),
        });
    }

    #[test]
    #[should_panic(expected = "Exceeds max duration")]
    fn dr_new_arg_settlement_time_exceed() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());

        contract.dr_new(bob(), 100, NewDataRequestArgs{
            sources: Vec::new(),
            outcomes: None,
            settlement_time: 1_000_000_000_000 * 1000 * 1000,
            challenge_period: 1500,
            target_contract: target(),
        });
    }

    #[test]
    #[should_panic(expected = "Validity bond not reached")]
    fn dr_new_not_enough_amount() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());

        contract.dr_new(bob(), 90, NewDataRequestArgs{
            sources: Vec::new(),
            outcomes: None,
            settlement_time: 0,
            challenge_period: 1500,
            target_contract: target(),
        });
    }

    #[test]
    fn dr_new_success_exceed_amount() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());

        let amount : Balance = contract.dr_new(bob(), 200, NewDataRequestArgs{
            sources: Vec::new(),
            outcomes: None,
            settlement_time: 0,
            challenge_period: 1500,
            target_contract: target(),
        });
        assert_eq!(amount, 100);
    }

    #[test]
    fn dr_new_success() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());

        let amount : Balance = contract.dr_new(bob(), 100, NewDataRequestArgs{
            sources: Vec::new(),
            outcomes: None,
            settlement_time: 0,
            challenge_period: 1500,
            target_contract: target(),
        });
        assert_eq!(amount, 0);
    }

    fn dr_new(contract : &mut Contract) {
        contract.dr_new(bob(), 100, NewDataRequestArgs{
            sources: Vec::new(),
            outcomes: Some(vec!["a".to_string(), "b".to_string()].to_vec()),
            settlement_time: 0,
            challenge_period: 1500,
            target_contract: target(),
        });
    }

    #[test]
    #[should_panic(expected = "Only the stake token contract can call this function")]
    fn dr_stake_non_stake_token() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());
        dr_new(&mut contract);

        testing_env!(get_context(alice()));
        contract.dr_stake(alice(),100,  StakeDataRequestArgs{
            id: U64(0),
            outcome: data_request::Outcome::Answer("42".to_string())
        });
    }

    #[test]
    #[should_panic(expected = "DataRequest with this id does not exist")]
    fn dr_stake_not_existing() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());
        contract.dr_stake(alice(),100,  StakeDataRequestArgs{
            id: U64(0),
            outcome: data_request::Outcome::Answer("42".to_string())
        });
    }

    #[test]
    #[should_panic(expected = "Incompatible outcome")]
    fn dr_stake_incompatible_answer() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());
        dr_new(&mut contract);

        contract.dr_stake(alice(),100,  StakeDataRequestArgs{
            id: U64(0),
            outcome: data_request::Outcome::Answer("42".to_string())
        });
    }

    #[test]
    #[should_panic(expected = "Can't stake in finalized DataRequest")]
    fn dr_stake_finalized_market() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());
        dr_new(&mut contract);

        contract.dr_stake(alice(), 200, StakeDataRequestArgs{
            id: U64(0),
            outcome: data_request::Outcome::Answer("a".to_string())
        });

        let mut ct : VMContext = get_context(token());
        ct.block_timestamp = 1501;
        testing_env!(ct);

        contract.dr_finalize(U64(0));

        contract.dr_stake(alice(), 200, StakeDataRequestArgs{
            id: U64(0),
            outcome: data_request::Outcome::Answer("b".to_string())
        });
    }


    #[test]
    #[should_panic(expected = "Invalid outcome list either exceeds min of: 2 or max of 8")]
    fn dr_stake_finalized_settlement_time() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());

        contract.dr_new(bob(), 100, NewDataRequestArgs{
            sources: Vec::new(),
            outcomes: Some(vec!["a".to_string()].to_vec()),
            settlement_time: 1,
            challenge_period: 1500,
            target_contract: target(),
        });

        contract.dr_stake(alice(), 200, StakeDataRequestArgs{
            id: U64(0),
            outcome: data_request::Outcome::Answer("a".to_string())
        });
    }

    #[test]
    fn dr_stake_success_partial() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());
        dr_new(&mut contract);

        let b : Balance = contract.dr_stake(alice(), 5, StakeDataRequestArgs{
            id: U64(0),
            outcome: data_request::Outcome::Answer("a".to_string())
        });
        assert_eq!(b, 0, "Invalid balance");

        let request : DataRequest = contract.data_requests.get(0).unwrap();
        assert_eq!(request.resolution_windows.len(), 1);


        let round0 : ResolutionWindow = request.resolution_windows.get(0).unwrap();
        assert_eq!(round0.round, 0);
        assert_eq!(round0.end_time, 1500);
        assert_eq!(round0.bond_size, 200);
    }

    #[test]
    fn dr_stake_success_full_at_t0() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());
        dr_new(&mut contract);

        let b : Balance = contract.dr_stake(alice(), 200, StakeDataRequestArgs{
            id: U64(0),
            outcome: data_request::Outcome::Answer("a".to_string())
        });
        assert_eq!(b, 0, "Invalid balance");

        let request : DataRequest = contract.data_requests.get(0).unwrap();
        assert_eq!(request.resolution_windows.len(), 2);

        let round0 : ResolutionWindow = request.resolution_windows.get(0).unwrap();
        assert_eq!(round0.round, 0);
        assert_eq!(round0.end_time, 1500);
        assert_eq!(round0.bond_size, 200);

        let round1 : ResolutionWindow = request.resolution_windows.get(1).unwrap();
        assert_eq!(round1.round, 1);
        assert_eq!(round1.end_time, 1000);
        assert_eq!(round1.bond_size, 400);
    }

    #[test]
    fn dr_stake_success_overstake_at_t600() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());
        dr_new(&mut contract);

        let mut ct : VMContext = get_context(token());
        ct.block_timestamp = 600;
        testing_env!(ct);

        let b : Balance = contract.dr_stake(alice(), 300, StakeDataRequestArgs{
            id: U64(0),
            outcome: data_request::Outcome::Answer("a".to_string())
        });
        assert_eq!(b, 100, "Invalid balance");

        let request : DataRequest = contract.data_requests.get(0).unwrap();
        assert_eq!(request.resolution_windows.len(), 2);

        let round0 : ResolutionWindow = request.resolution_windows.get(0).unwrap();
        assert_eq!(round0.round, 0);
        assert_eq!(round0.end_time, 2100);
        assert_eq!(round0.bond_size, 200);

        let round1 : ResolutionWindow = request.resolution_windows.get(1).unwrap();
        assert_eq!(round1.round, 1);
        assert_eq!(round1.end_time, 1600);
        assert_eq!(round1.bond_size, 400);
    }

    #[test]
    #[should_panic(expected = "Can only be finalized by final arbitrator")]
    fn dr_finalize_final_arb() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut c: oracle_config::OracleConfig = config();
        c.final_arbitrator_invoke_amount = 150;
        let mut contract = Contract::new(whitelist, c);
        dr_new(&mut contract);

        contract.dr_stake(alice(), 200, StakeDataRequestArgs{
            id: U64(0),
            outcome: data_request::Outcome::Answer("a".to_string())
        });

        contract.dr_finalize(U64(0));
    }

    #[test]
    #[should_panic(expected = "No resolution windows found, DataRequest not processed")]
    fn dr_finalize_no_resolutions() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());
        dr_new(&mut contract);

        contract.dr_finalize(U64(0));
    }

    #[test]
    #[should_panic(expected = "Challenge period not ended")]
    fn dr_finalize_active_challenge() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());
        dr_new(&mut contract);

        contract.dr_stake(alice(), 200, StakeDataRequestArgs{
            id: U64(0),
            outcome: data_request::Outcome::Answer("a".to_string())
        });

        contract.dr_finalize(U64(0));
    }

    #[test]
    fn dr_finalize_success() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());
        dr_new(&mut contract);

        contract.dr_stake(alice(), 200, StakeDataRequestArgs{
            id: U64(0),
            outcome: data_request::Outcome::Answer("a".to_string())
        });

        let mut ct : VMContext = get_context(token());
        ct.block_timestamp = 1501;
        testing_env!(ct);

        contract.dr_finalize(U64(0));

        let request : DataRequest = contract.data_requests.get(0).unwrap();
        assert_eq!(request.resolution_windows.len(), 2);
        assert_eq!(request.finalized_outcome.unwrap(), data_request::Outcome::Answer("a".to_string()));
    }

    #[test]
    #[should_panic(expected = "Outcome is incompatible for this round")]
    fn dr_stake_same_outcome() {
        testing_env!(get_context(token()));
        let whitelist = Some(vec![to_valid(bob()), to_valid(carol())]);
        let mut contract = Contract::new(whitelist, config());
        dr_new(&mut contract);

        contract.dr_stake(alice(), 300, StakeDataRequestArgs{
            id: U64(0),
            outcome: data_request::Outcome::Answer("a".to_string())
        });

        contract.dr_stake(alice(), 500, StakeDataRequestArgs{
            id: U64(0),
            outcome: data_request::Outcome::Answer("a".to_string())
        });
    }
}

// TODO single outcome test
