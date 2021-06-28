use crate::utils::*;
use oracle::whitelist::CustomFeeStakeArgs;

// Scenario: Alice (final arbitrator) creates a data request; Bob stakes on a
// correct outcome, and Carol stakes on an incorrect outcome. Bob and Carol
// take turns maxing out the bond size until final arbitrator is triggered.
// Alice finalizes, Bob and Carol collect the correct rewards.
#[test]
fn dr_scenario_1() {

    // configure test options and create data request
    let init_res = TestUtils::init(Some(TestSetupArgs {
        custom_fee: CustomFeeStakeArgs::None,
        validity_bond: 1,
        final_arbitrator_invoke_amount: 2500
    }));
    //let init_balance_alice = init_res.alice.get_token_balance(None);
    let _new_dr_res = init_res.alice.dr_new();
    // println!("{:?}", new_dr_res);    
    let dr_exist = init_res.alice.dr_exists(0);
    assert!(dr_exist, "something went wrong during dr creation");

    for i in 2..12 {
        println!("Round {}", i-1);
        let stake_amount = 2u128.pow(i) * 10u128.pow(24);
        println!("Bond size: {}", stake_amount);
        // even numbers => Bob stakes on correct outcome
        // odd numbers => Carol stakes on incorrect outcome
        match i % 2 == 0 {
            true => {
                let outcome_to_stake = data_request::Outcome::Answer(data_request::AnswerType::String("test".to_string()));
                println!("Bob balance before staking:   {}", init_res.bob.get_token_balance(None));
                let _res = init_res.bob.stake(0, outcome_to_stake, stake_amount);
                println!("Bob balance after staking:    {}", init_res.bob.get_token_balance(None));
            },
            false => {
                let outcome_to_stake = data_request::Outcome::Answer(data_request::AnswerType::String("test_wrong".to_string()));
                println!("Carol balance before staking: {}", init_res.bob.get_token_balance(None));
                let _res = init_res.carol.stake(0, outcome_to_stake, stake_amount);
                println!("Carol balance after staking:  {}", init_res.bob.get_token_balance(None));
            }
        };
    }

    // round 4
    //let result = init_res.carol.stake(0, wrong_outcome.clone(), (validity_bond.pow(5)) * 10u128.pow(24));

    //println!("{:?}", result);

    // let _post_stake_balance_oracle = init_res.alice.get_token_balance(Some(ORACLE_CONTRACT_ID.to_string()));
    // let post_stake_balance_alice = init_res.alice.get_token_balance(None);
    //assert_eq!(post_stake_balance_alice, init_balance_alice - stake_cost - dr_cost);
    
    init_res.alice.finalize(0);
    // init_res.alice.claim(0);
    
    //let post_claim_balance_alice = init_res.alice.get_token_balance(None);
    //assert_eq!(post_claim_balance_alice, init_balance_alice);
}
