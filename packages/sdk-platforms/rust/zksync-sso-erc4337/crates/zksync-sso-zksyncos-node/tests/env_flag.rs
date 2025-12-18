use std::env;

#[test]
#[ignore = "manual test"]
fn prints_test_node_backend_flag() {
    let value = env::var("SSO_TEST_NODE_BACKEND")
        .unwrap_or_else(|_| "<unset>".to_string());
    println!("SSO_TEST_NODE_BACKEND={value}");
}
