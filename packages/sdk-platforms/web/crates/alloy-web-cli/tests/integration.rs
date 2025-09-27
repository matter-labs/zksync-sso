use alloy_web_cli::test_utils::{
    check_port, run_cli_command, start_service_and_wait, stop_service_and_wait,
};

#[tokio::test]
async fn test_full_flow() {
    // Clean everything - stop any existing services
    if check_port(8545) {
        println!("Existing Anvil detected, stopping it...");
        let _ = stop_service_and_wait(&["infra", "anvil", "stop"], 8545, 5).await;
    }
    if check_port(4337) {
        println!("Existing bundler detected, stopping it...");
        let _ = stop_service_and_wait(&["infra", "bundler", "stop"], 4337, 5).await;
    }

    // Start all infrastructure and wait for services to be ready
    println!("Starting all infrastructure...");
    let start_result = start_service_and_wait(
        &["infra", "all", "start"],
        8545, // Check Anvil port as primary indicator
        "Infrastructure started successfully",
        20, // Longer timeout for full infrastructure
    )
    .await;

    match start_result {
        Ok((stdout, stderr)) => {
            println!("Infrastructure started: {}\n{}", stdout, stderr);

            // Check status
            let status = run_cli_command(&["infra", "status"]);
            if let Ok((stdout, stderr)) = status {
                println!("Status:\n{}\n{}", stdout, stderr);
            }

            // Check ports
            println!("Anvil running: {}", check_port(8545));
            println!("Bundler running: {}", check_port(4337));

            // Try account deployment
            let account_result = run_cli_command(&[
                "account",
                "deploy",
                "--factory",
                "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
            ]);

            if let Ok((stdout, stderr)) = account_result {
                println!("Account deployment:\n{}\n{}", stdout, stderr);
            }
        }
        Err(e) => {
            println!("Failed to start infrastructure: {}", e);
        }
    }

    // Clean up and wait for services to stop
    println!("Stopping all services...");
    let _ = stop_service_and_wait(&["infra", "all", "stop"], 8545, 10).await;

    // Verify services are stopped
    assert!(!check_port(8545), "Anvil should not be running after stop");
    assert!(
        !check_port(4337),
        "Bundler should not be running after stop"
    );
}
