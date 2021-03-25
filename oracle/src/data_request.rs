use crate::*;
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::json_types::{ U64 };
use near_sdk::serde::{ Deserialize, Serialize };
use near_sdk::{ env, Balance, AccountId };
use near_sdk::collections::{ Vector, LookupMap };

use crate::types::{ Timestamp, Duration };

const PERCENTAGE_DIVISOR: u16 = 10_000;

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
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
            end_time: env::block_timestamp() + challenge_period,
            bond_size: prev_bond * 2,
            outcome_to_stake: LookupMap::new(format!("ots{}:{}", dr_id, round).as_bytes().to_vec()),
            user_to_outcome_to_stake: LookupMap::new(format!("utots{}:{}", dr_id, round).as_bytes().to_vec()),
            bonded_outcome: None
        }   
    }

    // @returns amount to refund users because it was not staked
    fn stake(&mut self, sender: AccountId, answer: Outcome, amount: Balance) -> Balance {
        let stake_on_outcome = self.outcome_to_stake.get(&answer).unwrap_or(0);

        0
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
        window.stake(sender, answer, amount)
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

    // Challenge answer is used for the following scenario
    //     e.g.
    //     t = 0, challenge X is active
    //     t = 1, user send challenger transaction
    //     t = 2, challenge X is resolved, challenge Y is active
    //     t = 3, user TX is processed (stakes on wrong answer)
    /// Users can stake for a data request once (or they should unstake if thats possible)
    /// If the DRI has any predefined outcomes, the answers should be one of the predefined ones
    /// If the DRI does not have predefined outcomes, users can vote on answers freely
    /// The total stake is tracked, this stake get's divided amoung stakers with the most populair answer on finalization
    pub fn dr_stake(&mut self, sender: AccountId, amount: Balance, payload: StakeDataRequestArgs) -> Balance {
        self.assert_stake_token();
        let mut dr = self.dr_get_expect(payload.id.into());
        dr.assert_valid_answer(&payload.answer);
        dr.assert_not_finalized();
        dr.assert_settlement_time_passed();
        let _tvl = dr.get_tvl();

        dr.stake(sender, payload.answer, amount);

        // let mut round: DataRequestRound = dri.rounds.iter().last().unwrap();
        // if dri.rounds.len() > 1 {
        //     assert!(*round.outcomes.iter().next().unwrap() == payload.answer);
        // }
        // assert!(round.start_date > env::block_timestamp(), "NOT STARTED");
        // assert!(round.quorum_date == 0, "ALREADY PASSED");

        // round.total += amount;

        // let new_outcome_stake: Balance = match round.outcome_stakes.get_mut(&payload.answer) {
        //     Some(v) => {
        //         *v += amount;
        //         *v
        //     },
        //     None => {
        //         round.outcome_stakes.insert(payload.answer.clone(), amount);
        //         amount
        //     }
        // };
        // if new_outcome_stake > round.quorum_amount {
        //     round.quorum_date = env::block_timestamp();
        // }

        // let user_entries: &mut HashMap<String, Balance> = match round.user_outcome_stake.get_mut(&env::predecessor_account_id()) {
        //     Some(v) => {
        //         v
        //     }
        //     None => {
        //         round.user_outcome_stake.insert(env::predecessor_account_id(), HashMap::default());
        //         round.user_outcome_stake.get_mut(&env::predecessor_account_id()).unwrap()
        //     }
        // };

        // match user_entries.get_mut(&payload.answer) {
        //     Some(v) => {
        //         *v += amount;
        //     }
        //     None => {
        //         user_entries.insert(payload.answer, amount);
        //     }
        // }

        // TODO: return unspent tokens
        0
    }
}

impl Contract {
    fn dr_get_expect(&self, id: U64) -> DataRequest {
        self.data_requests.get(id.into()).expect("DataRequest with this id does not exist")
    }    
} 