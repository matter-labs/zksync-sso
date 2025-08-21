pub use crate::client::session::client::session_client::SessionClient;

#[cfg(test)]
mod tests {
    use crate::{
        client::{
            modular_account::{
                DeployModularAccountArgs, SessionModuleArgs,
                deploy_modular_account,
            },
            session::{
                actions::session::state::{
                    GetSessionStateArgs, get_session_state,
                },
                client::session_client::SessionClient,
            },
        },
        config::deploy_wallet::DeployWallet,
        utils::{
            alloy::extensions::ProviderExt,
            session::session_lib::session_spec::{
                SessionSpec, limit_type::LimitType,
                transfer_spec::TransferSpec, usage_limit::UsageLimit,
            },
            test_utils::{
                spawn_node_and_deploy_contracts,
                zksync_wallet_from_anvil_zksync,
            },
        },
    };
    use alloy::{
        network::{ReceiptResponse, TransactionBuilder},
        primitives::{U256, address},
        providers::Provider,
        signers::local::PrivateKeySigner,
    };
    use alloy_zksync::{
        network::transaction_request::TransactionRequest,
        provider::zksync_provider, wallet::ZksyncWallet,
    };
    use std::str::FromStr;

    #[tokio::test]
    async fn test_api_session_send_integration() -> eyre::Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;

        let mut config = config.clone();
        let node_url = &config.node_url;

        let (wallet, _, _) = zksync_wallet_from_anvil_zksync(&anvil_zksync)?;

        let wallet_address = wallet.default_signer().address();
        println!("XDB - Wallet address: {wallet_address}");

        let contracts = config.clone().contracts;

        println!("\n=");
        println!("=== SESSION SEND TEST - RUST REPLICATION ===");
        println!("=");

        // Private key for test account (WITH FUNDS) - Using rich wallet 4
        let private_key = "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a";
        let private_key_bytes = alloy::hex::decode(private_key)?;
        println!("private_key_bytes: {private_key_bytes:?}");

        // Owner private key for smart account (using rich wallet 3)
        let owner_private_key = "0x5de4111afa1a4b94908f83103c3a57e0c3c9e9da2dd5a02a84e9fde30d7e96c3";
        let owner_private_key_bytes = alloy::hex::decode(owner_private_key)?;
        println!("owner_private_key_bytes: {owner_private_key_bytes:?}");

        // Deterministic test values from TypeScript test
        let transfer_session_target =
            address!("0xdeBbD4CE2Bd6BD869D3ac93666A0D5F4fc06FC72");
        let session_owner_private_key = "0x6954ddb21936036ccad688e2770846f15380a721bfab26c6e531e25b35cb5971";
        let session_owner_private_key_bytes =
            alloy::hex::decode(session_owner_private_key)?;
        println!(
            "session_owner_private_key_bytes: {session_owner_private_key_bytes:?}"
        );

        let session_owner_address =
            address!("0x9BbC92a33F193174bf6Cc09c4b4055500d972479");
        let derived_session_owner_address = {
            let session_owner_signer =
                PrivateKeySigner::from_str(session_owner_private_key)?;
            let session_owner_wallet = ZksyncWallet::from(session_owner_signer);

            session_owner_wallet.default_signer().address()
        };
        println!(
            "derived_session_owner_address: {derived_session_owner_address:?}"
        );
        eyre::ensure!(
            derived_session_owner_address == session_owner_address,
            "derived_session_owner_address does not match session_owner_address, \n\t\texpected: {:?},\n\t\treceived: {:?}",
            session_owner_address,
            derived_session_owner_address
        );

        let expires_at = 1767225600u64; // January 1, 2026, 00:00:00 UTC

        // Update config with private key for test account
        config.deploy_wallet =
            Some(DeployWallet { private_key_hex: private_key.to_string() });

        // Create wallets and providers
        let wallet_client_signer = PrivateKeySigner::from_str(private_key)?;
        let wallet_client_wallet =
            ZksyncWallet::from(wallet_client_signer.clone());
        let wallet_client_address =
            wallet_client_wallet.default_signer().address();
        println!("wallet_client_address: {wallet_client_address:?}");

        let owner_signer = PrivateKeySigner::from_str(owner_private_key)?;
        let owner_wallet = ZksyncWallet::from(owner_signer);
        let owner_address = owner_wallet.default_signer().address();
        println!("owner_address: {owner_address:?}");

        println!("Test account address: {wallet_client_address}");
        println!("Account owner address: {owner_address}");

        // Create public provider
        let public_provider = zksync_provider()
            .with_recommended_fillers()
            .on_http(node_url.clone());

        // Create session configuration
        let session_config = SessionSpec {
            signer: session_owner_address,
            expires_at: U256::from(expires_at),
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(100000000000000000u64), // 0.1 ETH
                period: U256::from(0),
            },
            call_policies: vec![],
            transfer_policies: vec![TransferSpec {
                target: transfer_session_target,
                max_value_per_use: U256::from(10000000000000000u64), // 0.01 ETH
                value_limit: UsageLimit {
                    limit_type: LimitType::Unlimited,
                    limit: U256::from(0),
                    period: U256::from(0),
                },
            }],
        };

        println!("\n=== STEP 1: Deploy Account with Initial Session ===");
        println!("Account owner address: {owner_address}");

        let deploy_result = deploy_modular_account(
            DeployModularAccountArgs {
                owners: vec![owner_address],
                install_no_data_modules: vec![],
                session_module: Some(SessionModuleArgs {
                    location: contracts.session,
                    initial_session: Some(session_config.clone()),
                }),
                paymaster: None,
                passkey_module: None,
                unique_account_id: Some("session-send-test-001".to_string()),
            },
            &config,
        )
        .await?;

        let deployed_account_address = deploy_result.address;
        println!("Account deployed at: {deployed_account_address}");
        println!("Unique account ID: {:?}", deploy_result.unique_account_id);
        println!(
            "Transaction hash: {}",
            deploy_result.transaction_receipt.transaction_hash()
        );

        eyre::ensure!(
            deploy_result.transaction_receipt.status(),
            "Deployment transaction failed"
        );

        // Verify initial session state
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

        // Fund the deployed account
        println!("\n=== STEP 3: Fund the Deployed Account ===");
        let funding_amount = U256::from(1000000000000000000u64); // 1 ETH

        // Create funding provider
        let funding_provider = {
            let signer = PrivateKeySigner::from_str(private_key)?;
            let wallet = ZksyncWallet::from(signer);
            zksync_provider()
                .with_recommended_fillers()
                .wallet(wallet)
                .on_http(node_url.clone())
        };

        // Send funding transaction
        let funding_tx = {
            let tx_request = TransactionRequest::default()
                .with_to(deployed_account_address)
                .with_value(funding_amount);
            funding_provider.send_transaction(tx_request).await?
        };

        println!("Funding transaction sent: {}", funding_tx.tx_hash());

        let funding_receipt = funding_provider
            .wait_for_transaction_receipt(funding_tx.tx_hash().to_owned())
            .await?;

        println!(
            "Funding transaction confirmed: {:?}",
            funding_receipt.status()
        );

        let account_balance =
            public_provider.get_balance(deployed_account_address).await?;
        println!(
            "Smart account balance: {:.6} ETH",
            f64::from(account_balance) / 1e18
        );
        let expected_account_balance = U256::from(1000000000000000000u64);
        eyre::ensure!(
            account_balance == expected_account_balance,
            "Smart account should have balance, \n\t\texpected: {:?},\n\t\treceived: {:?}",
            expected_account_balance,
            account_balance
        );

        // Send transaction using session
        println!("\n=== STEP 4: Send Transaction Using Session ===");
        let transfer_amount = U256::from(5000000000000000u64); // 0.005 ETH

        // Check balances before transfer
        let sender_balance_before =
            public_provider.get_balance(deployed_account_address).await?;
        let receiver_balance_before =
            public_provider.get_balance(transfer_session_target).await?;

        println!("Balances before transfer:");
        println!(
            "  Sender: {:.6} ETH",
            f64::from(sender_balance_before) / 1e18
        );
        println!(
            "  Receiver: {:.6} ETH",
            f64::from(receiver_balance_before) / 1e18
        );

        // Create session client
        let session_key = {
            let session_key_bytes =
                alloy::hex::decode(session_owner_private_key)?;
            println!("session_key_bytes: {session_key_bytes:?}");

            let expected_session_key_bytes_from_hex = alloy::hex::decode(
                "0x6954ddb21936036ccad688e2770846f15380a721bfab26c6e531e25b35cb5971",
            )?;

            println!(
                "expected_session_key_bytes_from_hex: {expected_session_key_bytes_from_hex:?}"
            );

            let expected_session_key_bytes = vec![
                105, 84, 221, 178, 25, 54, 3, 108, 202, 214, 136, 226, 119, 8,
                70, 241, 83, 128, 167, 33, 191, 171, 38, 198, 229, 49, 226, 91,
                53, 203, 89, 113,
            ];

            eyre::ensure!(
                expected_session_key_bytes_from_hex
                    == expected_session_key_bytes,
                "Session key bytes should match expected value, \n\t\texpected: {:?},\n\t\treceived: {:?}",
                expected_session_key_bytes,
                expected_session_key_bytes_from_hex
            );

            eyre::ensure!(
                session_key_bytes == expected_session_key_bytes,
                "Session key bytes should be 32 bytes, \n\t\texpected: {:?},\n\t\treceived: {:?}",
                expected_session_key_bytes,
                session_key_bytes
            );

            alloy::primitives::FixedBytes::<32>::from_slice(&session_key_bytes)
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

        // Send the transaction
        let session_tx_request = TransactionRequest::default()
            .with_to(transfer_session_target)
            .with_value(transfer_amount);

        let session_transaction_receipt =
            session_client.send_transaction(session_tx_request).await?;

        println!("Transaction confirmed:");
        println!("  Status: {:?}", session_transaction_receipt.status());
        println!(
            "  Transaction hash: {}",
            session_transaction_receipt.transaction_hash()
        );
        println!("  Gas used: {:?}", session_transaction_receipt.gas_used());

        eyre::ensure!(
            session_transaction_receipt.status(),
            "Session transaction should succeed"
        );

        // Verify balances after transfer
        println!("\n=== STEP 5: Verify Balances After Transfer ===");
        let sender_balance_after =
            public_provider.get_balance(deployed_account_address).await?;
        let receiver_balance_after =
            public_provider.get_balance(transfer_session_target).await?;

        println!("Balances after transfer:");
        println!("  Sender: {:.6} ETH", f64::from(sender_balance_after) / 1e18);
        println!(
            "  Receiver: {:.6} ETH",
            f64::from(receiver_balance_after) / 1e18
        );

        let transfer_difference =
            receiver_balance_after - receiver_balance_before;
        println!(
            "Transfer amount verified: {:.6} ETH",
            f64::from(transfer_difference) / 1e18
        );
        eyre::ensure!(
            transfer_difference == transfer_amount,
            "Transfer amount should match expected value"
        );

        // The sender balance should have decreased by transfer amount + gas fees
        let sender_balance_decrease =
            sender_balance_before - sender_balance_after;
        println!(
            "Sender balance decreased by: {:.6} ETH (includes gas)",
            f64::from(sender_balance_decrease) / 1e18
        );
        eyre::ensure!(
            sender_balance_decrease > transfer_amount,
            "Sender balance should decrease by more than transfer amount (due to gas)"
        );

        // Verify session state after transaction
        println!("\n=== STEP 6: Verify Session State After Transaction ===");
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

        // Session should still be active
        eyre::ensure!(
            session_state_after_tx.session_state.status.is_active(),
            "Session should still be active"
        );

        // Fees remaining should have decreased
        let fees_used = initial_session_state.session_state.fees_remaining
            - session_state_after_tx.session_state.fees_remaining;
        println!("Fees used: {fees_used} wei");
        eyre::ensure!(fees_used > U256::from(0), "Fees should have been used");

        // Check transfer value remaining
        if !session_state_after_tx.session_state.transfer_value.is_empty() {
            let transfer_value_remaining =
                session_state_after_tx.session_state.transfer_value[0]
                    .remaining;
            println!("Transfer value remaining: {transfer_value_remaining}");

            // Just verify that some value remains (since it's unlimited)
            eyre::ensure!(
                transfer_value_remaining > U256::from(0),
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
        println!("4. Verified all balances updated correctly");
        println!("5. Verified session remains active with updated state");

        drop(anvil_zksync);

        Ok(())
    }
}
