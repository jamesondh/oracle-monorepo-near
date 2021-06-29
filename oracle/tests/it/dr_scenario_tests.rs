use crate::utils::*;
use oracle::whitelist::CustomFeeStakeArgs;

// Scenario: Bob stakes correctly and Carol takes turns (incorrectly) disputing
// until the final arbitrator is invoked and Alice must finalize
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
    let _new_dr_res = init_res.alice.dr_new();
    let dr_exist = init_res.alice.dr_exists(0);
    assert!(dr_exist, "something went wrong during dr creation");
    
    println!("Bob balance before staking:   {}", init_balance_bob);
    println!("Carol balance before staking: {}", init_balance_carol);
    
    for i in 0..14 {
        let bond_size = 2u128.pow(i+2) * 10u128.pow(24); // stake 2, 4, 16, 32, ...
        // even numbers => Bob stakes on correct outcome
        // odd numbers => Carol stakes on incorrect outcome
        match (i+2) % 2 == 0 {
            true => {
                println!("Round {}, bond size: {}, staking correctly with Bob", i, bond_size);
                let outcome_to_stake = data_request::Outcome::Answer(data_request::AnswerType::String("test".to_string()));
                let _res = init_res.bob.stake(0, outcome_to_stake, bond_size);
            },
            false => {
                println!("Round {}, bond size: {}, staking incorrectly with Carol", i, bond_size);
                let outcome_to_stake = data_request::Outcome::Answer(data_request::AnswerType::String("test_wrong".to_string()));
                let _res = init_res.carol.stake(0, outcome_to_stake, bond_size);
            }
        };
    }
    
    // get balances before finalization and claim
    let pre_claim_balance_bob = init_res.bob.get_token_balance(None);
    let pre_claim_balance_carol = init_res.carol.get_token_balance(None);
    
    let correct_outcome = data_request::Outcome::Answer(data_request::AnswerType::String("test".to_string()));
    init_res.alice.dr_final_arbitrator_finalize(0, correct_outcome);
    init_res.bob.claim(0);
    init_res.carol.claim(0);
    
    // get balances and differences from after staking/before claiming and before staking
    let post_balance_alice = init_res.alice.get_token_balance(None);
    let post_balance_bob = init_res.bob.get_token_balance(None);
    let post_balance_carol = init_res.carol.get_token_balance(None);
    let post_stake_difference_bob = post_balance_bob - pre_claim_balance_bob;
    let post_stake_difference_carol = post_balance_carol - pre_claim_balance_carol;
    let post_total_difference_bob = post_balance_bob - init_balance_bob;
    let post_total_difference_carol = init_balance_carol - post_balance_carol;
    let post_total_difference_alice = init_balance_alice - post_balance_alice;

    println!("Bob gained {} from claim for a total profit of {}", post_stake_difference_bob, post_total_difference_bob);
    println!("Carol gained {} from claim for a total loss of {}", post_stake_difference_carol, post_total_difference_carol);
    println!("Alice lost {} altogether", post_total_difference_alice);

    println!("Alice final balance:  {}", post_balance_alice);
    println!("Bob final balance:    {}", post_balance_bob);
    println!("Carol final balance:  {}", post_balance_carol);
    
}

// Scenario: Bob, Carol, and Jasper work together fill each resolution window
// (1/4 Bob, 1/4 Carol, 1/2 Jasper) while Peter escalates with a wrong staked
// outcome
#[test]
fn dr_scenario_2() {
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
    let init_balance_jasper = init_res.jasper.get_token_balance(None);
    let init_balance_peter = init_res.peter.get_token_balance(None);
    let _new_dr_res = init_res.alice.dr_new();
    let dr_exist = init_res.alice.dr_exists(0);
    assert!(dr_exist, "something went wrong during dr creation");
    
    println!("Bob balance before staking:    {}", init_balance_bob);
    println!("Carol balance before staking:  {}", init_balance_carol);
    println!("Jasper balance before staking: {}", init_balance_jasper);
    
    for i in 0..7 {
        let bond_size = 2u128.pow(i+2) * 10u128.pow(24); // stake 2, 4, 16, 32, ...
        // even numbers => Bob, Carol, Jasper stake on correct outcome
        // odd numbers => Peter stakes on incorrect outcome
        match (i+2) % 2 == 0 {
            true => {
                println!("Round {}, bond size: {}, staking correctly with Bob, Carol, Jasper", i, bond_size);
                let quarter_of_bond = bond_size / 4;
                let outcome_to_stake = data_request::Outcome::Answer(data_request::AnswerType::String("test".to_string()));
                let _res = init_res.bob.stake(0, outcome_to_stake.clone(), quarter_of_bond);
                let _res = init_res.carol.stake(0, outcome_to_stake.clone(), quarter_of_bond);
                let _res = init_res.jasper.stake(0, outcome_to_stake, quarter_of_bond*2);
            },
            false => {
                println!("Round {}, bond size: {}, staking incorrectly with Peter", i, bond_size);
                let outcome_to_stake = data_request::Outcome::Answer(data_request::AnswerType::String("test_wrong".to_string()));
                let _res = init_res.carol.stake(0, outcome_to_stake, bond_size);
            }
        };
    }
    
    // get balances before finalization and claim
    let pre_claim_balance_bob = init_res.bob.get_token_balance(None);
    let pre_claim_balance_carol = init_res.carol.get_token_balance(None);
    let pre_claim_balance_jasper = init_res.jasper.get_token_balance(None);
    let pre_claim_balance_peter = init_res.peter.get_token_balance(None);
    
    init_res.alice.finalize(0);
    init_res.bob.claim(0);
    init_res.carol.claim(0);
    init_res.jasper.claim(0);
    init_res.peter.claim(0);
    
    // get balances and differences from after staking/before claiming and before staking
    let post_balance_alice = init_res.alice.get_token_balance(None);
    let post_balance_bob = init_res.bob.get_token_balance(None);
    let post_balance_carol = init_res.carol.get_token_balance(None);
    let post_balance_jasper = init_res.jasper.get_token_balance(None);
    let post_balance_peter = init_res.peter.get_token_balance(None);
    let post_stake_difference_bob = post_balance_bob - pre_claim_balance_bob;
    let post_stake_difference_carol = post_balance_carol - pre_claim_balance_carol;
    let post_stake_difference_jasper = post_balance_jasper - pre_claim_balance_jasper;
    let post_stake_difference_peter = pre_claim_balance_peter - post_balance_peter;
    let post_total_difference_bob = post_balance_bob - init_balance_bob;
    let post_total_difference_carol = init_balance_carol - post_balance_carol;
    let post_total_difference_jasper = post_balance_jasper - init_balance_jasper;
    let post_total_difference_peter = init_balance_peter - post_balance_peter;
    let post_total_difference_alice = init_balance_alice - post_balance_alice;

    println!("Bob gained {} from claim for a total profit of {}", post_stake_difference_bob, post_total_difference_bob);
    println!("Carol gained {} from claim for a total loss of {}", post_stake_difference_carol, post_total_difference_carol);
    println!("Jasper gained {} from claim for a total profit of {}", post_stake_difference_jasper, post_total_difference_jasper);
    println!("Peter gained {} from claim for a total profit of {}", post_stake_difference_peter, post_total_difference_peter);
    println!("Alice lost {} altogether", post_total_difference_alice);

    println!("Alice final balance:  {}", post_balance_alice);
    println!("Bob final balance:    {}", post_balance_bob);
    println!("Carol final balance:  {}", post_balance_carol);
    println!("Jasper final balance: {}", post_balance_jasper);
    println!("Peter final balance:  {}", post_balance_peter);
    
}
