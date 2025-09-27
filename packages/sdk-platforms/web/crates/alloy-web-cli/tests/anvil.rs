use alloy_web_cli::test_utils::{
    check_port, rpc_call, run_cli_command, start_service_and_wait, stop_service_and_wait,
};

#[tokio::test]
async fn test_anvil_start_stop() {
    println!("Test starting...");

    // Stop any existing Anvil instance if it's running
    if check_port(8545) {
        println!("Existing Anvil detected, stopping it...");
        let _ = stop_service_and_wait(&["infra", "anvil", "stop"], 8545, 5).await;
    }

    // Start Anvil and wait for it to be ready
    println!("Starting Anvil...");
    let result = start_service_and_wait(
        &["infra", "anvil", "start"],
        8545,
        "Anvil started successfully",
        10,
    )
    .await;
    println!("Start result: {:?}", result);
    assert!(result.is_ok(), "Failed to start Anvil: {:?}", result);

    // Check if Anvil is running
    assert!(check_port(8545), "Anvil should be running on port 8545");

    // Make RPC call to verify Anvil is working
    let rpc_result = rpc_call(8545, "eth_chainId");
    assert!(rpc_result.is_ok(), "RPC call failed: {:?}", rpc_result);
    assert!(
        rpc_result.unwrap().contains("0x7a69"),
        "Unexpected chain ID"
    );

    // Check status
    let status = run_cli_command(&["infra", "status"]);
    assert!(status.is_ok(), "Failed to get status: {:?}", status);

    // Stop Anvil and wait for it to actually stop
    let stop_result = stop_service_and_wait(&["infra", "anvil", "stop"], 8545, 5).await;
    assert!(
        stop_result.is_ok(),
        "Failed to stop Anvil: {:?}",
        stop_result
    );

    // Verify it's stopped
    assert!(!check_port(8545), "Anvil should not be running after stop");
}
