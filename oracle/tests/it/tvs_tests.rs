use crate::utils::*;

#[test]
fn fetch_tvs_it_test() {
    let registrees = Some(
        vec![
            oracle::whitelist::RegistryEntry{
                code_base_url: Some("test".to_string()),
                interface_name: "requestor".to_string(),
                contract_entry: REQUEST_INTERFACE_CONTRACT_ID.to_string(),
            },
            oracle::whitelist::RegistryEntry{
                code_base_url: Some("test".to_string()),
                interface_name: "requestor".to_string(),
                contract_entry: REQUEST_INTERFACE_CONTRACT_ID.to_string(),
            },
        ]
    );
    let init_res = TestUtils::init(registrees);
    let tvs_res = init_res.alice.fetch_tvs();
    println!("{:?}", tvs_res);
}
