use crate::*;
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::json_types::{ U64 };
use near_sdk::serde::{ Deserialize, Serialize };
use near_sdk::{ env, Balance, AccountId };
use near_sdk::collections::{ Vector, LookupMap };

use crate::types::{ Timestamp, Duration };

const PERCENTAGE_DIVISOR: u16 = 10_000;

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize, Debug)]
pub enum Outcome {
    Answer(String),
    Invalid
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
    fn stake(&mut self, sender: AccountId, answer: Outcome, amount: Balance) -> Balance;
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
    fn stake(&mut self, sender: AccountId, answer: Outcome, amount: Balance) -> Balance {
        let mut stake_on_outcome = self.outcome_to_stake.get(&answer).unwrap_or(0);
        let mut user_to_outcomes = self.user_to_outcome_to_stake
            .get(&sender)
            .unwrap_or(LookupMap::new(format!("utots:{}:{}:{}", self.dr_id, self.round, sender))); 
        let mut user_stake_on_outcome = user_to_outcomes.get(&answer).unwrap_or(0);

        let stake_open = self.bond_size - stake_on_outcome;
        let unspent = if amount > stake_open {
            amount - stake_open
        } else {
            0
        };

        let staked = amount - unspent;

        let new_stake_on_outcome = stake_on_outcome + staked;
        self.outcome_to_stake.insert(&answer, &new_stake_on_outcome);
        
        let new_user_stake_on_outcome = user_stake_on_outcome + staked;
        user_to_outcomes.insert(&answer, &new_stake_on_outcome);
        self.user_to_outcome_to_stake.insert(&sender, &user_to_outcomes);

        // If this stake fills the bond set final answer which will trigger a new resolution_window to be created
        if new_stake_on_outcome == self.bond_size {
            self.bonded_outcome = Some(answer);
        }

        unspent
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
}

trait DataRequestChange {
    fn new(sender: AccountId, id: u64, config: oracle_config::OracleConfig, request_data: NewDataRequestArgs) -> Self;
    fn stake(&mut self, sender: AccountId, answer: Outcome, amount: Balance) -> Balance;
    fn finalize(&mut self);
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
        } 
    }

    // @returns amount of tokens that didn't get staked
    fn stake(&mut self, sender: AccountId, answer: Outcome, amount: Balance) -> Balance {
        let mut window = self.resolution_windows
            .iter()
            .last()
            .unwrap_or(
                ResolutionWindow::new(self.id, 0, self.calc_resolution_bond(), self.initial_challenge_period)
            );

        let unspent = window.stake(sender, answer, amount);

        // If first window push it to vec, else replace updated window struct
        if self.resolution_windows.len() == 0 {
            self.resolution_windows.push(&window);
        } else {
            self.resolution_windows.replace(
                self.resolution_windows.len() - 1, // Last window
                &window
            );
        }

        // Check if this stake closed the current window, if so create next window
        if window.bonded_outcome.is_some() {
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

    fn finalize(&mut self) {
        self.finalized_outcome = self.get_final_outcome();
    }
}

trait DataRequestView {
    fn assert_valid_answer(&self, answer: &Outcome);
    fn assert_not_finalized(&self);
    fn assert_settlement_time_passed(&self);
    fn assert_can_finalize(&self);
    fn get_final_outcome(&self) -> Option<Outcome>;
    fn get_tvl(&self) -> Balance;
    fn calc_fee(&self) -> Balance;
    fn calc_resolution_bond(&self) -> Balance;
    fn calc_validity_bond_to_return(&self) -> Balance;
    fn calc_resolution_fee_payout(&self) -> Balance;
}

impl DataRequestView for DataRequest {
    fn assert_valid_answer(&self, answer: &Outcome) {
        match &self.outcomes {
            Some(outcomes) => match answer {
                Outcome::Answer(answer) => assert!(outcomes.contains(&answer), "Incompatible answer"),
                Outcome::Invalid => ()
            },
            None => ()
        }
    }

    fn assert_not_finalized(&self) {
        assert!(self.finalized_outcome.is_none(), "Can't stake in finalized market");
    }

    fn assert_settlement_time_passed(&self) {
        assert!(env::block_timestamp() >= self.settlement_time, "Data request cannot be processed yet");
    }

    fn assert_can_finalize(&self) {
        let last_window = self.resolution_windows.iter().last().expect("No resolutions found, DataRequest not processed");
        self.assert_not_finalized();
        assert!(env::block_timestamp() >= last_window.end_time, "Challenge period not ended");
    }

    fn get_final_outcome(&self) -> Option<Outcome> {
        let last_bonded_window_i = self.resolution_windows.len() - 2; // Last window after end_time never has a bonded outcome
        let last_bonded_window = self.resolution_windows.get(last_bonded_window_i).unwrap();
        last_bonded_window.bonded_outcome
    }

    fn get_tvl(&self) -> Balance {
        self.requestor.get_tvl(self.id.into()).into()
    }


    // TODO: rewrite resolution / stake bond logic
    fn calc_fee(&self) -> Balance { 
        let tvl = self.get_tvl();
        self.config.resolution_fee_percentage as Balance * tvl / PERCENTAGE_DIVISOR as Balance
    }

    /**
     * @notice Calculates the size of the resolution bond. If the accumulated fee is smaller than the validity bond, we payout the validity bond to validators, thus they have to stake double in order to be 
     * eligible for the reward, in the case that the fee is greater than the validity bond validators need to have a cumulative stake of double the fee amount
     * @returns The size of the initial `resolution_bond` denominated in `stake_token`
     */
    fn calc_resolution_bond(&self) -> u128 {
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
    fn calc_validity_bond_to_return(&self) -> u128 {
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
    fn calc_resolution_fee_payout(&self) -> u128 {
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

impl Contract {
    pub fn dr_finalize(&mut self, id: U64) {
        let mut dr = self.dr_get_expect(id);
        dr.assert_can_finalize();
        dr.finalize();
        self.data_requests.replace(id.into(), &dr);
        // TODO: Return validity bond to creator
    }

    // Merge config and payload
    pub fn dr_new(&mut self, sender: AccountId, amount: Balance, payload: NewDataRequestArgs) -> Balance {
        self.assert_whitelisted(sender.to_string());
        self.assert_bond_token();
        self.dr_validate(&payload);
        assert!(amount >= self.config.validity_bond);

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

    pub fn dr_stake(&mut self, sender: AccountId, amount: Balance, payload: StakeDataRequestArgs) -> Balance {
        self.assert_stake_token();
        let mut dr = self.dr_get_expect(payload.id.into());
        dr.assert_valid_answer(&payload.answer);
        dr.assert_not_finalized();
        dr.assert_settlement_time_passed();
        let _tvl = dr.get_tvl();

        let unspent_stake = dr.stake(sender, payload.answer, amount);

        self.data_requests.replace(payload.id.into(), &dr);

        unspent_stake
    }
}

impl Contract {
    fn dr_get_expect(&self, id: U64) -> DataRequest {
        self.data_requests.get(id.into()).expect("DataRequest with this id does not exist")
    }    
} 