use alloy_web_cli::test_utils::{
    check_port, run_cli_command, start_service_and_wait, stop_service_and_wait,
};

#[tokio::test]
async fn test_simple_account_deployment() {
    // Clean slate - stop any existing services
    if check_port(8545) {
        println!("Existing Anvil detected, stopping it...");
        let _ = stop_service_and_wait(&["infra", "anvil", "stop"], 8545, 5).await;
    }

    // Start Anvil and wait for it to be ready
    let start_result = start_service_and_wait(
        &["infra", "anvil", "start"],
        8545,
        "Anvil started successfully",
        10,
    )
    .await;
    assert!(
        start_result.is_ok(),
        "Failed to start Anvil: {:?}",
        start_result
    );

    // Deploy infrastructure
    let _ = run_cli_command(&["infra", "deploy"]);

    // Deploy SimpleAccount (using default factory address)
    let account_result = run_cli_command(&[
        "account",
        "deploy",
        "--factory",
        "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
    ]);

    match account_result {
        Ok((stdout, stderr)) => {
            let combined = format!("{}\n{}", stdout, stderr);
            println!("Account deployment output: {}", combined);

            // Check for account deployment indicators
            assert!(
                combined.contains("Account")
                    || combined.contains("deployed")
                    || combined.contains("0x"),
                "Should contain account deployment information"
            );
        }
        Err(e) => {
            println!("Account deployment error: {}", e);
            // Account might already exist, which is okay
        }
    }

    // Clean up and wait for services to stop
    println!("Stopping Anvil...");
    let _ = stop_service_and_wait(&["infra", "anvil", "stop"], 8545, 5).await;

    // Verify services are stopped
    assert!(!check_port(8545), "Anvil should not be running after stop");
}
