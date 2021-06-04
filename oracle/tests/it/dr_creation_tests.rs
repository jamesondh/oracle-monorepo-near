use crate::utils::*;

#[test]
fn dr_new_test() {
    let init_res = TestUtils::init();
    init_res.alice.dr_new();
}