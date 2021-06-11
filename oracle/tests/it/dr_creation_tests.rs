use crate::utils::*;

#[test]
fn dr_new_test() {
    let init_res = TestUtils::init();
    let new_res = init_res.alice.dr_new();
    println!("dr {:?}", new_res);
    let dr_exist = init_res.alice.dr_exists(0);
    assert!(dr_exist, "something went wrong during dr creation");
}