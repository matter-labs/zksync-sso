use super::super::constants::{
    SESSION_SEND_TEST_EXPECTED_SESSION_KEY_BYTES, SESSION_SEND_TEST_EXPIRES_AT,
    SESSION_SEND_TEST_FUNDING_AMOUNT_U64, SESSION_SEND_TEST_OWNER_ADDRESS,
    SESSION_SEND_TEST_OWNER_PRIVATE_KEY, SESSION_SEND_TEST_PRIVATE_KEY,
    SESSION_SEND_TEST_SESSION_OWNER_ADDRESS,
    SESSION_SEND_TEST_SESSION_OWNER_PRIVATE_KEY,
    SESSION_SEND_TEST_TRANSFER_AMOUNT, SESSION_SEND_TEST_TRANSFER_AMOUNT_U64,
    SESSION_SEND_TEST_TRANSFER_TARGET, SESSION_SEND_TEST_UNIQUE_ACCOUNT_ID,
    create_session_send_test_config_json,
};
use crate::config::ConfigLoader;
use sdk::{
    api::{
        account::{
            fund::fund_account,
            modular_account::{
                DeployModularAccountArgs, SessionModuleArgs,
                deploy_modular_account,
            },
            session::{
                client::SessionClient,
                state::{GetSessionStateArgs, get_session_state},
            },
            transaction::Transaction,
        },
        utils::{decode_fixed_bytes_hex, parse_address, u256_from},
    },
    config::deploy_wallet::DeployWallet,
};

pub async fn send_transaction() -> eyre::Result<()> {
    println!(
        "Running holistic send transaction test (matching test_api_session_send_integration)..."
    );

    let mut config = ConfigLoader::load()?;

    let private_key = SESSION_SEND_TEST_PRIVATE_KEY;
    let owner_private_key = SESSION_SEND_TEST_OWNER_PRIVATE_KEY;
    let session_owner_private_key = SESSION_SEND_TEST_SESSION_OWNER_PRIVATE_KEY;

    config.deploy_wallet =
        Some(DeployWallet { private_key_hex: private_key.to_string() });

    let transfer_session_target =
        parse_address(SESSION_SEND_TEST_TRANSFER_TARGET)?;
    let session_owner_address =
        parse_address(SESSION_SEND_TEST_SESSION_OWNER_ADDRESS)?;
    let owner_address = parse_address(SESSION_SEND_TEST_OWNER_ADDRESS)?;

    let expires_at = SESSION_SEND_TEST_EXPIRES_AT;

    println!("Test configuration:");
    println!("  Private key: {private_key}");
    println!("  Owner private key: {owner_private_key}");
    println!("  Session owner private key: {session_owner_private_key}");
    println!("  Transfer target: {transfer_session_target}");
    println!("  Session owner address: {session_owner_address}");
    println!("  Owner address: {owner_address}");
    println!("  Expires at: {expires_at}");

    let session_config_json = create_session_send_test_config_json();

    let session_config =
        sdk::api::account::session::session_lib::session_spec_from_json(
            &session_config_json,
        )?;

    println!("\n=== STEP 1: Deploy Account with Initial Session ===");
    println!("Account owner address: {owner_address}");

    let deploy_result = deploy_modular_account(
        DeployModularAccountArgs {
            owners: vec![owner_address],
            install_no_data_modules: vec![],
            session_module: Some(SessionModuleArgs {
                location: config.contracts.session,
                initial_session: Some(session_config.clone()),
            }),
            paymaster: None,
            passkey_module: None,
            unique_account_id: Some(
                SESSION_SEND_TEST_UNIQUE_ACCOUNT_ID.to_string(),
            ),
        },
        &config,
    )
    .await?;

    let deployed_account_address = deploy_result.address;
    println!("Account deployed at: {deployed_account_address}");
    println!("Unique account ID: {:?}", deploy_result.unique_account_id);
    println!("Transaction receipt: {}", deploy_result.transaction_receipt_json);

    println!("\n=== STEP 2: Verify Initial Session State ===");
    let initial_session_state = get_session_state(
        GetSessionStateArgs {
            account: deployed_account_address,
            session_config: session_config.clone(),
        },
        &config,
    )
    .await?;

    println!("Initial session state:");
    println!(
        "  Status: {:?} (1 = Active)",
        initial_session_state.session_state.status
    );
    println!(
        "  Fees remaining: {:?}",
        initial_session_state.session_state.fees_remaining
    );

    eyre::ensure!(
        initial_session_state.session_state.is_active(),
        "Initial session should be active"
    );
    eyre::ensure!(
        initial_session_state.session_state.fees_remaining
            == session_config.fee_limit.limit,
        "Fee limit should match configured value"
    );

    println!("\n=== STEP 3: Fund the Deployed Account ===");
    let funding_amount = u256_from(SESSION_SEND_TEST_FUNDING_AMOUNT_U64);

    fund_account(deployed_account_address, funding_amount, &config).await?;
    println!("Account funded successfully!");

    println!("\n=== STEP 4: Send Transaction Using Session ===");
    let transfer_amount = u256_from(SESSION_SEND_TEST_TRANSFER_AMOUNT_U64);

    let session_key = {
        let session_key_bytes =
            decode_fixed_bytes_hex::<32>(session_owner_private_key)?;
        println!("Session key bytes: {:?}", session_key_bytes.as_slice());

        let expected_session_key_bytes =
            SESSION_SEND_TEST_EXPECTED_SESSION_KEY_BYTES.to_vec();

        println!(
            "Expected session key bytes: {:?}",
            expected_session_key_bytes.as_slice()
        );

        eyre::ensure!(
            session_key_bytes.as_slice()
                == expected_session_key_bytes.as_slice(),
            "Session key bytes should match expected value from test"
        );

        println!("✓ Session key bytes match expected value from test");

        session_key_bytes
    };

    let session_client = SessionClient::new(
        deployed_account_address,
        session_key,
        session_config.clone(),
        config.clone(),
    )?;

    println!("Sending transaction...");
    println!("  From: {deployed_account_address}");
    println!("  To: {transfer_session_target}");
    println!("  Amount: {:.6} ETH", f64::from(transfer_amount) / 1e18);

    let transaction = Transaction {
        from: deployed_account_address,
        to: Some(transfer_session_target),
        value: Some(SESSION_SEND_TEST_TRANSFER_AMOUNT.to_string()), // 0.005 ETH in wei
        input: None,
    };

    let session_transaction_receipt =
        session_client.send_transaction(transaction.try_into()?).await?;

    println!("Transaction confirmed successfully!");
    println!("Transaction receipt: {session_transaction_receipt:?}");

    println!("\n=== STEP 5: Verify Session State After Transaction ===");
    let session_state_after_tx = get_session_state(
        GetSessionStateArgs {
            account: deployed_account_address,
            session_config: session_config.clone(),
        },
        &config,
    )
    .await?;

    println!("Session state after transaction:");
    println!(
        "  Status: {:?} (1 = Active)",
        session_state_after_tx.session_state.status
    );
    println!(
        "  Fees remaining: {:?}",
        session_state_after_tx.session_state.fees_remaining
    );

    eyre::ensure!(
        session_state_after_tx.session_state.status.is_active(),
        "Session should still be active"
    );

    let fees_used = initial_session_state.session_state.fees_remaining
        - session_state_after_tx.session_state.fees_remaining;
    println!("Fees used: {fees_used} wei");
    eyre::ensure!(fees_used > u256_from(0u64), "Fees should have been used");

    if !session_state_after_tx.session_state.transfer_value.is_empty() {
        let transfer_value_remaining =
            session_state_after_tx.session_state.transfer_value[0].remaining;
        println!("Transfer value remaining: {transfer_value_remaining}");

        // Just verify that some value remains (since it's unlimited)
        eyre::ensure!(
            transfer_value_remaining > u256_from(0u64),
            "Transfer value should remain available"
        );
    }

    println!("\n=");
    println!("✅ SESSION SEND TEST COMPLETED SUCCESSFULLY ✅");
    println!("{}", "=".repeat(80));
    println!("Test Summary:");
    println!("1. Deployed smart account with initial session");
    println!("2. Funded the account with 1 ETH");
    println!("3. Sent 0.005 ETH using session key");
    println!("4. Verified session remains active with updated state");

    Ok(())
}

pub async fn send_transaction_with_account(
    account_address: &str,
) -> eyre::Result<()> {
    println!(
        "Running send transaction test for specific account: {account_address}"
    );

    // For now, just call the main function since we're making it holistic
    // In the future, this could be modified to work with a pre-deployed account
    send_transaction().await
}
