use crate::utils::*;
use oracle::whitelist::CustomFeeStakeArgs;

// Scenario: Bob and Carol take turns staking the exact bond size each round. Bob
// ends the escalation game on a correct outcome
#[test]
fn dr_scenario_1() {
    // configure test options and create data request
    let validity_bond = 1;
    let init_res = TestUtils::init(Some(TestSetupArgs {
        custom_fee: CustomFeeStakeArgs::None,
        validity_bond,
        final_arbitrator_invoke_amount: 2500
    }));
    let init_balance_alice = init_res.alice.get_token_balance(None);
    let init_balance_bob = init_res.bob.get_token_balance(None);
    let init_balance_carol = init_res.carol.get_token_balance(None);
    println!("Alice balance before data request creation: {}", init_balance_alice);
    let _new_dr_res = init_res.alice.dr_new();
    let dr_exist = init_res.alice.dr_exists(0);
    assert!(dr_exist, "something went wrong during dr creation");
    
    println!("Alice balance after data request creation:  {}", init_res.alice.get_token_balance(None));
    println!("Bob balance before staking:                 {}", init_balance_bob);
    println!("Carol balance before staking:               {}", init_balance_carol);
    
    for i in 0..11 {
        let stake_amount = 2u128.pow(i+2) * 10u128.pow(24); // stake 2, 4, 16, 32, ...
        // even numbers => Bob stakes on correct outcome
        // odd numbers => Carol stakes on incorrect outcome
        match (i+2) % 2 == 0 {
            true => {
                println!("Round {}, bond size: {}, staking correctly with Bob", i, stake_amount);
                let outcome_to_stake = data_request::Outcome::Answer(data_request::AnswerType::String("test".to_string()));
                let _res = init_res.bob.stake(0, outcome_to_stake, stake_amount);
            },
            false => {
                println!("Round {}, bond size: {}, staking incorrectly with Carol", i, stake_amount);
                let outcome_to_stake = data_request::Outcome::Answer(data_request::AnswerType::String("test_wrong".to_string()));
                let _res = init_res.carol.stake(0, outcome_to_stake, stake_amount);
            }
        };
    }
    
    // get balances before finalization and claim
    let pre_claim_balance_bob = init_res.bob.get_token_balance(None);
    let pre_claim_balance_carol = init_res.carol.get_token_balance(None);
    
    init_res.alice.finalize(0);
    init_res.bob.claim(0);
    init_res.carol.claim(0);
    
    // get balances and differences from after staking/before claiming and before staking
    let post_balance_alice = init_res.alice.get_token_balance(None);
    let post_balance_bob = init_res.bob.get_token_balance(None);
    let post_balance_carol = init_res.carol.get_token_balance(None);
    let post_stake_difference_bob = post_balance_bob - pre_claim_balance_bob;
    let post_stake_difference_carol = post_balance_carol - pre_claim_balance_carol;
    let post_total_difference_bob = init_balance_bob - pre_claim_balance_bob;
    let post_total_difference_carol = init_balance_carol - pre_claim_balance_carol;
    let post_total_difference_alice = init_balance_alice - post_balance_alice;

    println!("Bob gained {} from claim for a total profit of {}", post_stake_difference_bob, post_total_difference_bob);
    println!("Carol gained {} from claim for a total loss of {}", post_stake_difference_carol, post_total_difference_carol);
    println!("Alice lost {} altogether", post_total_difference_alice);

    println!("Alice final balance:  {}", post_balance_alice);
    println!("Bob final balance:    {}", post_balance_bob);
    println!("Carol final balance:  {}", post_balance_carol);
    
}
