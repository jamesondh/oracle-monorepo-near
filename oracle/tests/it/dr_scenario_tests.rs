use crate::utils::*;
use oracle::whitelist::CustomFeeStakeArgs;
use near_sdk::json_types::U128;
use oracle::data_request::PERCENTAGE_DIVISOR;

#[test]
fn dr_scenario_1() {

    let validity_bond: u128 = 1;

    let init_res = TestUtils::init(Some(TestSetupArgs {
        custom_fee: CustomFeeStakeArgs::None,
        validity_bond: Some(validity_bond)
    }));
    let init_balance_alice = init_res.alice.get_token_balance(None);

    let _res = init_res.alice.dr_new();
    let _post_new_balance_oracle = init_res.alice.get_token_balance(Some(ORACLE_CONTRACT_ID.to_string()));
    
    let dr_exist = init_res.alice.dr_exists(0);
    assert!(dr_exist, "something went wrong during dr creation");
    let outcome = data_request::Outcome::Answer(
        data_request::AnswerType::String("test".to_string())
    );

    // let pre_stake_balance_bob = init_res.bob.get_token_balance(None);
    // println!("{}", pre_stake_balance_bob);
    
    let _res = init_res.bob.stake(0, outcome.clone(), (validity_bond.pow(2)) * 10u128.pow(24));
    let _res = init_res.carol.stake(0, outcome.clone(), (validity_bond.pow(3)) * 10u128.pow(24));
    let _res = init_res.carol.stake(0, outcome.clone(), (validity_bond.pow(4)) * 10u128.pow(24));

    let _post_stake_balance_oracle = init_res.alice.get_token_balance(Some(ORACLE_CONTRACT_ID.to_string()));
    let post_stake_balance_alice = init_res.alice.get_token_balance(None);
    //assert_eq!(post_stake_balance_alice, init_balance_alice - stake_cost - dr_cost);
    
    init_res.alice.finalize(0);
    init_res.alice.claim(0);
    
    //let post_claim_balance_alice = init_res.alice.get_token_balance(None);
    //assert_eq!(post_claim_balance_alice, init_balance_alice);
}
