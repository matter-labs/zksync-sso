mod send_integration;

#[cfg(test)]
mod tests {
    use crate::{
        client::{
            modular_account::{
                DeployModularAccountArgs, SessionModuleArgs,
                deploy_modular_account,
            },
            session::{
                actions::session::{
                    create::{CreateSessionArgs, create_session},
                    hash::get_session_hash,
                    revoke::{RevokeSessionArgs, revoke_session},
                    send::sign_fn_from_signer,
                    state::{GetSessionStateArgs, get_session_state},
                },
                client::session_client::SessionClient,
            },
        },
        config::{
            Config, contracts::SSOContracts, deploy_wallet::DeployWallet,
        },
        contracts::SsoAccount,
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
        primitives::{FixedBytes, U256, address, hex, keccak256},
        providers::Provider,
        signers::local::PrivateKeySigner,
    };
    use alloy_zksync::{
        network::{
            transaction_request::TransactionRequest,
            unsigned_tx::eip712::PaymasterParams,
        },
        provider::zksync_provider,
        wallet::ZksyncWallet,
    };
    use eyre::{Ok, eyre};
    use std::str::FromStr;

    #[tokio::test]
    #[ignore = "this test is deterministic and should be run manually"]
    async fn test_create_session() -> eyre::Result<()> {
        println!(
            "\n=== RUST SDK REPLICATION OF 'should deploy proxy account via factory' TEST ==="
        );

        // Hardcoded deterministic configuration (no dynamic node/contract deployment)
        let expected_funding_signer_address =
            address!("0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65"); // Account: 0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65 (Rich Wallet 4)

        // Account: 0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65 (Rich Wallet 4: 0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65)
        let private_key = "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a";

        // Owner private key for ECDSA smart account client (using rich wallet 3: 0x90F79bf6EB2c4f870365E785982E1f101E93b906)
        let owner_private_key = "0x5de4111afa1a4b94908f83103c3a57e0c3c9e9da2dd5a02a84e9fde30d7e96c3";

        let node_url = url::Url::parse("http://127.0.0.1:8011")?;
        let config = Config {
            node_url: node_url.clone(),
            contracts: SSOContracts {
                account_factory: address!(
                    "0xf3463a972529039ac73db42b4a48997a6ea679cd"
                ),
                session: address!("0x027ba0517cfa4471457c6e74f201753d98e7431d"),
                passkey: address!("0xa472581ea2aca6e6bd8ea6cca95d3e1297aa5ae3"),
                account_paymaster: address!(
                    "0xd3de94e23314d43341be4103e508b75e070460ed"
                ),
                recovery: address!(
                    "0xe72da237538a1854535e9565dbee0b414f382698"
                ),
            },
            deploy_wallet: Some(DeployWallet {
                private_key_hex: private_key.to_string(),
            }),
        };

        let wallet_client_signer = PrivateKeySigner::from_str(private_key)?;
        let wallet_client_wallet =
            ZksyncWallet::from(wallet_client_signer.clone());
        let wallet_client_address =
            wallet_client_wallet.default_signer().address();
        println!("wallet_client_address: {wallet_client_address:?}");
        let expected_wallet_client_address =
            address!("0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65");
        eyre::ensure!(
            wallet_client_address == expected_wallet_client_address,
            "wallet_client_address does not match expected address, expected: {:?}, received: {:?}",
            expected_wallet_client_address,
            wallet_client_address
        );

        let owner_signer = PrivateKeySigner::from_str(owner_private_key)?;
        let owner_wallet = ZksyncWallet::from(owner_signer);
        let owner_address = owner_wallet.default_signer().address();
        println!("owner_address: {owner_address:?}");
        let expected_owner_address =
            address!("0x6a34Ea49c29BF7Cce95F51E7F0f419831Ad5dBC6");
        eyre::ensure!(
            owner_address == expected_owner_address,
            "owner_address does not match expected address, expected: {:?}, received: {:?}",
            expected_owner_address,
            owner_address
        );

        // Test configuration values matching the TypeScript test

        let transfer_session_target =
            address!("0xdeBbD4CE2Bd6BD869D3ac93666A0D5F4fc06FC72");
        let session_owner_address =
            address!("0x9BbC92a33F193174bf6Cc09c4b4055500d972479");
        let random_salt =
            keccak256("sdk-test-factory-replication-010".as_bytes()); // Unique ID for deterministic salt
        let expires_at = 1749040108u64;

        println!("=== REPLICATION DATA VERIFICATION ===");
        println!("transferSessionTarget: {transfer_session_target}");
        println!("sessionOwnerAddress: {session_owner_address}");
        println!("randomSalt: 0x{}", hex::encode(random_salt));
        println!("expiresAt: {expires_at}");
        println!("factoryContract: {}", config.contracts.account_factory);
        println!("sessionContract: {}", config.contracts.session);

        // Create provider for contract calls
        let public_provider = {
            let node_url: url::Url = config.clone().node_url;
            zksync_provider().with_recommended_fillers().on_http(node_url)
        };

        // Create the exact same session configuration as the original test
        let exact_session_config = SessionSpec {
            signer: session_owner_address,
            expires_at: U256::from(expires_at),
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(100000000000000000u64), // 0.1 ETH
                period: U256::from(0),
            },
            call_policies: vec![], // Empty array same as original
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

        println!("=== SESSION CONFIG VERIFICATION ===");
        println!("Session signer: {}", exact_session_config.signer);
        println!("Session expiresAt: {}", exact_session_config.expires_at);
        println!(
            "Fee limit type: {:?} (1 = Lifetime)",
            exact_session_config.fee_limit.limit_type
        );
        println!(
            "Fee limit amount: {} wei (0.1 ETH)",
            exact_session_config.fee_limit.limit
        );
        println!("Fee limit period: {}", exact_session_config.fee_limit.period);
        println!(
            "Call policies length: {}",
            exact_session_config.call_policies.len()
        );
        println!(
            "Transfer policies length: {}",
            exact_session_config.transfer_policies.len()
        );
        println!(
            "Transfer policy target: {}",
            exact_session_config.transfer_policies[0].target
        );
        println!(
            "Transfer policy maxValuePerUse: {} wei (0.01 ETH)",
            exact_session_config.transfer_policies[0].max_value_per_use
        );
        println!(
            "Transfer policy valueLimit type: {:?} (0 = Unlimited)",
            exact_session_config.transfer_policies[0].value_limit.limit_type
        );
        println!(
            "Transfer policy valueLimit amount: {}",
            exact_session_config.transfer_policies[0].value_limit.limit
        );
        println!(
            "Transfer policy valueLimit period: {}",
            exact_session_config.transfer_policies[0].value_limit.period
        );

        // Step 1: Deploy modular account WITH initial session
        println!(
            "\n--- Step 1: Deploying modular account with initial session (SDK equivalent) ---"
        );
        println!(
            "Deploying account with factory: {}",
            config.contracts.account_factory
        );
        println!(
            "Account deployer (fixtures.wallet.address equivalent): {wallet_client_address}"
        );
        println!("Initial session signer: {}", exact_session_config.signer);
        println!("Account owner address: {owner_address}");

        let deploy_result = deploy_modular_account(
            DeployModularAccountArgs {
                account_factory: config.contracts.account_factory,
                owners: vec![owner_address], // Use the ECDSA owner
                install_no_data_modules: vec![],
                session_module: Some(SessionModuleArgs {
                    location: config.contracts.session,
                    initial_session: Some(exact_session_config.clone()),
                }),
                paymaster: None,
                passkey_module: None,
                unique_account_id: Some(
                    "sdk-test-factory-replication-010".to_string(),
                ),
            },
            &config,
        )
        .await?;

        let deployed_account_address = deploy_result.address;
        println!("Account deployed successfully!");
        println!("  Deployed address: {deployed_account_address}");
        println!(
            "  Transaction hash: {}",
            deploy_result.transaction_receipt.transaction_hash()
        );
        println!("  Status: {:?}", deploy_result.transaction_receipt.status());

        // Verify deployment was successful
        if !deploy_result.transaction_receipt.status() {
            return Err(eyre!("Deployment transaction failed"));
        }

        // Step 2: Verify session module is a validator
        println!("\n--- Step 2: Verifying session module is a validator ---");

        let account_contract =
            SsoAccount::new(deployed_account_address, &public_provider);
        let is_module_validator = account_contract
            .isModuleValidator(config.contracts.session)
            .call()
            .await?
            ._0;

        println!("Session module is validator: {is_module_validator}");
        eyre::ensure!(
            is_module_validator,
            "Session module should be a validator"
        );

        // Step 3: Get initial session state
        println!("\n--- Step 3: Getting initial session state ---");
        let initial_session_state = get_session_state(
            GetSessionStateArgs {
                account: deployed_account_address,
                session_config: exact_session_config.clone(),
            },
            &config,
        )
        .await?;

        println!("Initial session state retrieved:");
        println!(
            "  Status: {:?} (1 = Active)",
            initial_session_state.session_state.status
        );
        println!(
            "  Fees remaining: {:?}",
            initial_session_state.session_state.fees_remaining
        );
        println!(
            "  Transfer value entries: {:?}",
            initial_session_state.session_state.transfer_value.len()
        );
        println!(
            "  Call value entries: {:?}",
            initial_session_state.session_state.call_value.len()
        );
        println!(
            "  Call params entries: {:?}",
            initial_session_state.session_state.call_params.len()
        );

        // Verify the session is active
        eyre::ensure!(
            initial_session_state.session_state.status.is_active(),
            "Initial session should be active (status=1)"
        );

        // Verify fee limit is set correctly
        eyre::ensure!(
            initial_session_state.session_state.fees_remaining
                == exact_session_config.fee_limit.limit,
            "Fee limit should match configured value"
        );

        // Verify transfer policies are set
        eyre::ensure!(
            initial_session_state.session_state.transfer_value.len() == 1,
            "Should have exactly one transfer policy"
        );

        if !initial_session_state.session_state.transfer_value.is_empty() {
            println!("Transfer value entry 0:");
            println!(
                "  Target: {}",
                initial_session_state.session_state.transfer_value[0].target
            );
            println!(
                "  Remaining: {}",
                initial_session_state.session_state.transfer_value[0].remaining
            );

            // Verify target matches our transfer session target
            eyre::ensure!(
                initial_session_state.session_state.transfer_value[0].target
                    == transfer_session_target,
                "Transfer target should match configured value"
            );
        }

        // Step 4: Calculate and verify session hash
        println!("\n--- Step 4: Calculating session hash ---");
        let session_hash = get_session_hash(exact_session_config.clone())?;
        println!("Session hash: 0x{}", hex::encode(session_hash.fixed_bytes()));

        let expected_session_hash: FixedBytes<32> = {
            let expected_session_hash = hex::decode(
                "c424e4a2319b9e449d85c13d6511e63eb383fb975dc68a96d5d7fcdcbbce675a",
            )?;
            FixedBytes::from_slice(&expected_session_hash)
        };
        eyre::ensure!(
            session_hash.fixed_bytes() == expected_session_hash,
            "Session hash does not match expected value"
        );

        // Verify session hash is deterministic and not empty
        let empty_hash = alloy::primitives::FixedBytes::<32>::from([0u8; 32]);
        eyre::ensure!(
            session_hash.fixed_bytes() != empty_hash,
            "Session hash should not be empty"
        );

        // Verify that calculating the hash again produces the same result
        let session_hash_2 = get_session_hash(exact_session_config.clone())?;
        eyre::ensure!(
            session_hash == session_hash_2,
            "Session hash should be deterministic"
        );

        println!(
            "Session hash verified as deterministic: 0x{}",
            hex::encode(session_hash.fixed_bytes())
        );

        // Step 5: Fund the smart account and test session revocation
        println!(
            "\n--- Step 5: Fund smart account and test session revocation ---"
        );

        // Fund the smart account for transaction fees (1 ETH)
        println!("Funding smart account for transaction fees...");
        let funding_amount = U256::from(1000000000000000000u64); // 1 ETH

        let funding_provider = {
            let node_url: url::Url = config.clone().node_url;
            let signer = PrivateKeySigner::from_str(private_key)?;
            let signer_address = signer.address();
            println!("signer_address: {signer_address:?}");

            eyre::ensure!(
                signer_address == expected_funding_signer_address,
                "signer address does not match owner address, expected: {:?}, received: {:?}",
                expected_funding_signer_address,
                signer_address
            );

            let wallet = ZksyncWallet::from(signer.clone());

            zksync_provider()
                .with_recommended_fillers()
                .wallet(wallet)
                .on_http(node_url)
        };

        // Send funding transaction to the smart account
        let funding_tx = {
            let tx_request = TransactionRequest::default()
                .with_to(deployed_account_address)
                .with_value(funding_amount);

            funding_provider.send_transaction(tx_request).await?
        };
        println!("Funding transaction sent: {}", funding_tx.tx_hash());

        // Wait for funding transaction to be confirmed
        let funding_receipt = funding_provider
            .wait_for_transaction_receipt(funding_tx.tx_hash().to_owned())
            .await?;
        println!(
            "Funding transaction confirmed: {:?}",
            funding_receipt.status()
        );

        // Check smart account balance
        let account_balance =
            public_provider.get_balance(deployed_account_address).await?;
        println!("Smart account balance: {account_balance} wei");
        println!(
            "Smart account balance: {:.6} ETH",
            f64::from(account_balance) / 1e18
        );
        let expected_account_balance = U256::from(1000000000000000000u64);
        eyre::ensure!(
            account_balance == expected_account_balance,
            "Smart account balance should be 1 ETH:\n    expected: {:?}\n    received: {:?}",
            expected_account_balance,
            account_balance
        );

        println!("  Smart account address: {deployed_account_address}");
        println!("  Using owner private key for revocation");
        println!(
            "  Session hash to revoke: 0x{}",
            hex::encode(session_hash.fixed_bytes())
        );

        // Revoke the initial session
        println!("Attempting to revoke session using owner's credentials...");

        let revoke_args = RevokeSessionArgs { session_id: session_hash };

        let signer = alloy::signers::local::PrivateKeySigner::from_str(
            owner_private_key,
        )?;
        let sign_fn = sign_fn_from_signer(signer);
        let revoke_result = revoke_session(
            revoke_args,
            deployed_account_address,
            sign_fn,
            &config,
        )
        .await?;

        println!("Session revocation successful:");
        println!(
            "  Transaction hash: {}",
            revoke_result.transaction_receipt.transaction_hash()
        );
        println!(
            "  Gas used: {:?}",
            revoke_result.transaction_receipt.gas_used()
        );
        println!("  Status: {:?}", revoke_result.transaction_receipt.status());

        eyre::ensure!(
            revoke_result.transaction_receipt.status(),
            "Revocation transaction should succeed"
        );

        // Step 6: Verify session is now revoked
        println!("\n--- Step 6: Verifying session is revoked ---");
        let revoked_session_state = get_session_state(
            GetSessionStateArgs {
                account: deployed_account_address,
                session_config: exact_session_config.clone(),
            },
            &config,
        )
        .await?;

        println!("Session state after revocation:");
        println!(
            "  Status: {:?} (2 = Closed/Revoked)",
            revoked_session_state.session_state.status
        );
        println!(
            "  Fees remaining: {:?}",
            revoked_session_state.session_state.fees_remaining
        );

        // Verify session is now closed/revoked (status = 2)
        eyre::ensure!(
            revoked_session_state.session_state.status.is_closed(),
            "Session should be closed/revoked (status=2)"
        );

        println!("✓ Session successfully revoked");

        // Step 7: Create a new session after revocation
        println!("\n--- Step 7: Creating a new session after revocation ---");

        // Create a second session configuration with different parameters for the transaction
        // Rich Wallet (3)
        let second_session_owner_private_key = owner_private_key; // Different key
        let second_session_owner_address =
            address!("90F79bf6EB2c4f870365E785982E1f101E93b906"); // ANOTHER ADDRESS NOT DERIVED FROM THE PRIVATE KEY ABOVE

        // Vitalik's address for the session transaction
        let vitalik_address =
            address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");

        let second_session_config = SessionSpec {
            signer: second_session_owner_address,
            expires_at: U256::from(1767225600u64), // January 1st, 2026 00:00:00 UTC
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(50000000000000000u64), // 0.05 ETH
                period: U256::from(0),
            },
            call_policies: vec![], // No call policies
            transfer_policies: vec![TransferSpec {
                target: vitalik_address, // Allow transfers to Vitalik's address
                max_value_per_use: U256::from(5000000000000000u64), // 0.005 ETH per transfer
                value_limit: UsageLimit {
                    limit_type: LimitType::Unlimited,
                    limit: U256::from(0),
                    period: U256::from(0),
                },
            }],
        };

        println!("Second session configuration:");
        println!("  Signer: {}", second_session_config.signer);
        println!("  Expires at: {}", second_session_config.expires_at);
        println!(
            "  Fee limit: {} wei (0.05 ETH)",
            second_session_config.fee_limit.limit
        );
        println!(
            "  Transfer max value per use: {} wei (0.005 ETH)",
            second_session_config.transfer_policies[0].max_value_per_use
        );

        // Create the session using the owner provider (simulating ECDSA client)
        let _second_session_key = {
            // Convert the hex string to FixedBytes<32>
            let private_key_bytes =
                hex::decode(second_session_owner_private_key)?;
            FixedBytes::<32>::from_slice(&private_key_bytes)
        };

        let create_session_args = CreateSessionArgs {
            account: deployed_account_address,
            session_config: second_session_config.clone(),
            paymaster: Some(PaymasterParams {
                paymaster: config.contracts.account_paymaster,
                paymaster_input: alloy::primitives::Bytes::new(),
            }),
        };

        println!("Creating second session using ECDSA-like client...");

        let second_session_signer =
            PrivateKeySigner::from_str(second_session_owner_private_key)?;
        let sign_fn = sign_fn_from_signer(second_session_signer);
        let second_session_result =
            create_session(create_session_args, sign_fn, &config).await?;

        println!("Second session created:");
        println!(
            "  Transaction hash: {}",
            second_session_result.transaction_receipt.transaction_hash()
        );
        println!(
            "  Status: {:?}",
            second_session_result.transaction_receipt.status()
        );

        eyre::ensure!(
            second_session_result.transaction_receipt.status(),
            "Second session creation should succeed"
        );

        // Check the status of the new session
        println!("\n--- Checking status of the new session ---");
        let second_session_hash =
            get_session_hash(second_session_config.clone())?;
        println!(
            "Second session hash: 0x{}",
            hex::encode(second_session_hash.fixed_bytes())
        );

        let second_session_state = get_session_state(
            GetSessionStateArgs {
                account: deployed_account_address,
                session_config: second_session_config.clone(),
            },
            &config,
        )
        .await?;

        println!("Second session state:");
        println!(
            "  Status: {:?} (1 = Active)",
            second_session_state.session_state.status
        );
        println!(
            "  Fees remaining: {:?}",
            second_session_state.session_state.fees_remaining
        );
        println!(
            "  Transfer value entries: {:?}",
            second_session_state.session_state.transfer_value.len()
        );

        // Verify the second session is active
        eyre::ensure!(
            second_session_state.session_state.status.is_active(),
            "Second session should be active"
        );
        eyre::ensure!(
            second_session_state.session_state.fees_remaining
                == second_session_config.fee_limit.limit,
            "Second session fee limit should match"
        );

        // Send a transaction using SessionClient to Vitalik's address
        println!("\n--- Sending session transaction using SessionClient ---");

        let transfer_amount = U256::from(3000000000000000u64); // 0.003 ETH - within our 0.005 ETH limit

        println!("Session transaction details:");
        println!("  From (smart account): {deployed_account_address}");
        println!("  To (Vitalik): {vitalik_address}");
        println!("  Amount: {transfer_amount} wei (0.003 ETH)");
        println!(
            "  Session max per use: {} wei (0.005 ETH)",
            second_session_config.transfer_policies[0].max_value_per_use
        );
        println!("  Session signer: {}", second_session_config.signer);

        // Check balances before transfer
        let balance_before_transfer =
            public_provider.get_balance(deployed_account_address).await?;
        println!(
            "Smart account balance before transfer: {balance_before_transfer} wei"
        );

        let vitalik_balance_before =
            public_provider.get_balance(vitalik_address).await?;
        println!(
            "Vitalik balance before transfer: {vitalik_balance_before} wei"
        );

        // Create SessionClient and send transaction
        let session_client = SessionClient::new(
            deployed_account_address,
            _second_session_key,
            second_session_config.clone(),
            config.clone(),
        )?;

        println!("SessionClient created:");
        println!("  Smart account address: {deployed_account_address}");
        println!("  Session key: 0x{}", hex::encode(_second_session_key));
        println!(
            "  Session signer from config: {}",
            second_session_config.signer
        );

        // Create the transaction request for the transfer
        let session_tx_request = TransactionRequest::default()
            .with_to(vitalik_address)
            .with_value(transfer_amount);

        println!("Sending transaction through SessionClient...");
        let session_transaction_receipt =
            session_client.send_transaction(session_tx_request).await?;

        println!("Session transaction confirmed:");
        println!("  Status: {:?}", session_transaction_receipt.status());
        println!(
            "  Transaction hash: {}",
            session_transaction_receipt.transaction_hash()
        );

        eyre::ensure!(
            session_transaction_receipt.status(),
            "Session transaction should succeed"
        );

        // Check balances after transaction
        let balance_after_transfer =
            public_provider.get_balance(deployed_account_address).await?;
        println!(
            "Smart account balance after transfer: {balance_after_transfer} wei"
        );

        let vitalik_balance_after =
            public_provider.get_balance(vitalik_address).await?;
        println!("Vitalik balance after transfer: {vitalik_balance_after} wei");

        // Verify the transfer happened
        let transfer_difference =
            vitalik_balance_after - vitalik_balance_before;
        println!("Transfer amount verified: {transfer_difference} wei");
        eyre::ensure!(
            transfer_difference == transfer_amount,
            "Transfer amount should match expected value"
        );

        // Check updated session state after transaction
        println!("\n--- Checking updated session state after transaction ---");
        let session_state_after_tx = get_session_state(
            GetSessionStateArgs {
                account: deployed_account_address,
                session_config: second_session_config.clone(),
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
        println!(
            "  Transfer value entries: {:?}",
            session_state_after_tx.session_state.transfer_value.len()
        );

        if !session_state_after_tx.session_state.transfer_value.is_empty() {
            println!(
                "Transfer value remaining: {}",
                session_state_after_tx.session_state.transfer_value[0]
                    .remaining
            );

            // The transfer value limit should have decreased by the transfer amount
            let original_transfer_limit_hex = "115792089237316195423570985008687907853269984665640564039457584007913129639935";
            let original_transfer_limit =
                U256::from_str(original_transfer_limit_hex)?;
            let expected_remaining = original_transfer_limit - transfer_amount;
            let actual_remaining =
                session_state_after_tx.session_state.transfer_value[0]
                    .remaining;
            println!("Expected remaining: {expected_remaining}");
            println!("Actual remaining: {actual_remaining}");
            eyre::ensure!(
                actual_remaining == expected_remaining,
                "Transfer value limit should have decreased by the transfer amount"
            );
        }

        // Session should still be active after the transaction
        eyre::ensure!(
            session_state_after_tx.session_state.status.is_active(),
            "Session should still be active after the transaction"
        );

        println!("Session transaction completed successfully! 🎉");

        Ok(())
    }

    #[tokio::test]
    async fn test_create_session_anvil() -> eyre::Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;

        let mut config = config.clone();
        // let node_url = &config.node_url;

        let (wallet, _, _) = zksync_wallet_from_anvil_zksync(&anvil_zksync)?;

        let wallet_address = wallet.default_signer().address();
        println!("XDB - Wallet address: {wallet_address}");

        // let contracts = config.clone().contracts;

        println!(
            "\n=== RUST SDK REPLICATION OF 'should deploy proxy account via factory' TEST ==="
        );

        // Hardcoded deterministic configuration (no dynamic node/contract deployment)
        let expected_funding_signer_address =
            address!("0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65"); // Account: 0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65 (Rich Wallet 4)

        // Account: 0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65 (Rich Wallet 4: 0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65)
        let private_key = "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a";

        // Owner private key for ECDSA smart account client (using rich wallet 3: 0x90F79bf6EB2c4f870365E785982E1f101E93b906)
        let owner_private_key = "0x5de4111afa1a4b94908f83103c3a57e0c3c9e9da2dd5a02a84e9fde30d7e96c3";

        config.deploy_wallet =
            Some(DeployWallet { private_key_hex: private_key.to_string() });

        let wallet_client_signer = PrivateKeySigner::from_str(private_key)?;
        let wallet_client_wallet =
            ZksyncWallet::from(wallet_client_signer.clone());
        let wallet_client_address =
            wallet_client_wallet.default_signer().address();
        println!("wallet_client_address: {wallet_client_address:?}");
        let expected_wallet_client_address =
            address!("0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65");
        eyre::ensure!(
            wallet_client_address == expected_wallet_client_address,
            "wallet_client_address does not match expected address, expected: {:?}, received: {:?}",
            expected_wallet_client_address,
            wallet_client_address
        );

        let owner_signer = PrivateKeySigner::from_str(owner_private_key)?;
        let owner_wallet = ZksyncWallet::from(owner_signer);
        let owner_address = owner_wallet.default_signer().address();
        println!("owner_address: {owner_address:?}");
        let expected_owner_address =
            address!("0x6a34Ea49c29BF7Cce95F51E7F0f419831Ad5dBC6");
        eyre::ensure!(
            owner_address == expected_owner_address,
            "owner_address does not match expected address, expected: {:?}, received: {:?}",
            expected_owner_address,
            owner_address
        );

        // Test configuration values matching the TypeScript test

        let transfer_session_target =
            address!("0xdeBbD4CE2Bd6BD869D3ac93666A0D5F4fc06FC72");
        let session_owner_address =
            address!("0x9BbC92a33F193174bf6Cc09c4b4055500d972479");
        let random_salt =
            keccak256("sdk-test-factory-replication-010".as_bytes()); // Unique ID for deterministic salt
        let expires_at = 1749040108u64;

        println!("=== REPLICATION DATA VERIFICATION ===");
        println!("transferSessionTarget: {transfer_session_target}");
        println!("sessionOwnerAddress: {session_owner_address}");
        println!("randomSalt: 0x{}", hex::encode(random_salt));
        println!("expiresAt: {expires_at}");
        println!("factoryContract: {}", config.contracts.account_factory);
        println!("sessionContract: {}", config.contracts.session);

        // Create provider for contract calls
        let public_provider = {
            let node_url: url::Url = config.clone().node_url;
            zksync_provider().with_recommended_fillers().on_http(node_url)
        };

        // Create the exact same session configuration as the original test
        let exact_session_config = SessionSpec {
            signer: session_owner_address,
            expires_at: U256::from(expires_at),
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(100000000000000000u64), // 0.1 ETH
                period: U256::from(0),
            },
            call_policies: vec![], // Empty array same as original
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

        println!("=== SESSION CONFIG VERIFICATION ===");
        println!("Session signer: {}", exact_session_config.signer);
        println!("Session expiresAt: {}", exact_session_config.expires_at);
        println!(
            "Fee limit type: {:?} (1 = Lifetime)",
            exact_session_config.fee_limit.limit_type
        );
        println!(
            "Fee limit amount: {} wei (0.1 ETH)",
            exact_session_config.fee_limit.limit
        );
        println!("Fee limit period: {}", exact_session_config.fee_limit.period);
        println!(
            "Call policies length: {}",
            exact_session_config.call_policies.len()
        );
        println!(
            "Transfer policies length: {}",
            exact_session_config.transfer_policies.len()
        );
        println!(
            "Transfer policy target: {}",
            exact_session_config.transfer_policies[0].target
        );
        println!(
            "Transfer policy maxValuePerUse: {} wei (0.01 ETH)",
            exact_session_config.transfer_policies[0].max_value_per_use
        );
        println!(
            "Transfer policy valueLimit type: {:?} (0 = Unlimited)",
            exact_session_config.transfer_policies[0].value_limit.limit_type
        );
        println!(
            "Transfer policy valueLimit amount: {}",
            exact_session_config.transfer_policies[0].value_limit.limit
        );
        println!(
            "Transfer policy valueLimit period: {}",
            exact_session_config.transfer_policies[0].value_limit.period
        );

        // Step 1: Deploy modular account WITH initial session
        println!(
            "\n--- Step 1: Deploying modular account with initial session (SDK equivalent) ---"
        );
        println!(
            "Deploying account with factory: {}",
            config.contracts.account_factory
        );
        println!(
            "Account deployer (fixtures.wallet.address equivalent): {wallet_client_address}"
        );
        println!("Initial session signer: {}", exact_session_config.signer);
        println!("Account owner address: {owner_address}");

        let deploy_result = deploy_modular_account(
            DeployModularAccountArgs {
                account_factory: config.contracts.account_factory,
                owners: vec![owner_address], // Use the ECDSA owner
                install_no_data_modules: vec![],
                session_module: Some(SessionModuleArgs {
                    location: config.contracts.session,
                    initial_session: Some(exact_session_config.clone()),
                }),
                paymaster: None,
                passkey_module: None,
                unique_account_id: Some(
                    "sdk-test-factory-replication-010".to_string(),
                ),
            },
            &config,
        )
        .await?;

        let deployed_account_address = deploy_result.address;
        println!("Account deployed successfully!");
        println!("  Deployed address: {deployed_account_address}");
        println!(
            "  Transaction hash: {}",
            deploy_result.transaction_receipt.transaction_hash()
        );
        println!("  Status: {:?}", deploy_result.transaction_receipt.status());

        // Verify deployment was successful
        if !deploy_result.transaction_receipt.status() {
            return Err(eyre!("Deployment transaction failed"));
        }

        // Step 2: Verify session module is a validator
        println!("\n--- Step 2: Verifying session module is a validator ---");

        let account_contract =
            SsoAccount::new(deployed_account_address, &public_provider);
        let is_module_validator = account_contract
            .isModuleValidator(config.contracts.session)
            .call()
            .await?
            ._0;

        println!("Session module is validator: {is_module_validator}");
        eyre::ensure!(
            is_module_validator,
            "Session module should be a validator"
        );

        // Step 3: Get initial session state
        println!("\n--- Step 3: Getting initial session state ---");
        let initial_session_state = get_session_state(
            GetSessionStateArgs {
                account: deployed_account_address,
                session_config: exact_session_config.clone(),
            },
            &config,
        )
        .await?;

        println!("Initial session state retrieved:");
        println!(
            "  Status: {:?} (1 = Active)",
            initial_session_state.session_state.status
        );
        println!(
            "  Fees remaining: {:?}",
            initial_session_state.session_state.fees_remaining
        );
        println!(
            "  Transfer value entries: {:?}",
            initial_session_state.session_state.transfer_value.len()
        );
        println!(
            "  Call value entries: {:?}",
            initial_session_state.session_state.call_value.len()
        );
        println!(
            "  Call params entries: {:?}",
            initial_session_state.session_state.call_params.len()
        );

        // Verify the session is active
        eyre::ensure!(
            initial_session_state.session_state.status.is_active(),
            "Initial session should be active (status=1)"
        );

        // Verify fee limit is set correctly
        eyre::ensure!(
            initial_session_state.session_state.fees_remaining
                == exact_session_config.fee_limit.limit,
            "Fee limit should match configured value"
        );

        // Verify transfer policies are set
        eyre::ensure!(
            initial_session_state.session_state.transfer_value.len() == 1,
            "Should have exactly one transfer policy"
        );

        if !initial_session_state.session_state.transfer_value.is_empty() {
            println!("Transfer value entry 0:");
            println!(
                "  Target: {}",
                initial_session_state.session_state.transfer_value[0].target
            );
            println!(
                "  Remaining: {}",
                initial_session_state.session_state.transfer_value[0].remaining
            );

            // Verify target matches our transfer session target
            eyre::ensure!(
                initial_session_state.session_state.transfer_value[0].target
                    == transfer_session_target,
                "Transfer target should match configured value"
            );
        }

        // Step 4: Calculate and verify session hash
        println!("\n--- Step 4: Calculating session hash ---");
        let session_hash = get_session_hash(exact_session_config.clone())?;
        println!("Session hash: 0x{}", hex::encode(session_hash.fixed_bytes()));

        let expected_session_hash: FixedBytes<32> = {
            let expected_session_hash = hex::decode(
                "c424e4a2319b9e449d85c13d6511e63eb383fb975dc68a96d5d7fcdcbbce675a",
            )?;
            FixedBytes::from_slice(&expected_session_hash)
        };
        eyre::ensure!(
            session_hash.fixed_bytes() == expected_session_hash,
            "Session hash does not match expected value"
        );

        // Verify session hash is deterministic and not empty
        let empty_hash = alloy::primitives::FixedBytes::<32>::from([0u8; 32]);
        eyre::ensure!(
            session_hash.fixed_bytes() != empty_hash,
            "Session hash should not be empty"
        );

        // Verify that calculating the hash again produces the same result
        let session_hash_2 = get_session_hash(exact_session_config.clone())?;
        eyre::ensure!(
            session_hash == session_hash_2,
            "Session hash should be deterministic"
        );

        println!(
            "Session hash verified as deterministic: 0x{}",
            hex::encode(session_hash.fixed_bytes())
        );

        // Step 5: Fund the smart account and test session revocation
        println!(
            "\n--- Step 5: Fund smart account and test session revocation ---"
        );

        // Fund the smart account for transaction fees (1 ETH)
        println!("Funding smart account for transaction fees...");
        let funding_amount = U256::from(1000000000000000000u64); // 1 ETH

        let funding_provider = {
            let node_url: url::Url = config.clone().node_url;
            let signer = PrivateKeySigner::from_str(private_key)?;
            let signer_address = signer.address();
            println!("signer_address: {signer_address:?}");

            eyre::ensure!(
                signer_address == expected_funding_signer_address,
                "signer address does not match owner address, expected: {:?}, received: {:?}",
                expected_funding_signer_address,
                signer_address
            );

            let wallet = ZksyncWallet::from(signer.clone());

            zksync_provider()
                .with_recommended_fillers()
                .wallet(wallet)
                .on_http(node_url)
        };

        // Send funding transaction to the smart account
        let funding_tx = {
            let tx_request = TransactionRequest::default()
                .with_to(deployed_account_address)
                .with_value(funding_amount);

            funding_provider.send_transaction(tx_request).await?
        };
        println!("Funding transaction sent: {}", funding_tx.tx_hash());

        // Wait for funding transaction to be confirmed
        let funding_receipt = funding_provider
            .wait_for_transaction_receipt(funding_tx.tx_hash().to_owned())
            .await?;
        println!(
            "Funding transaction confirmed: {:?}",
            funding_receipt.status()
        );

        // Check smart account balance
        let account_balance =
            public_provider.get_balance(deployed_account_address).await?;
        println!("Smart account balance: {account_balance} wei");
        println!(
            "Smart account balance: {:.6} ETH",
            f64::from(account_balance) / 1e18
        );
        let expected_account_balance = U256::from(1000000000000000000u64);
        eyre::ensure!(
            account_balance == expected_account_balance,
            "Smart account balance should be 1 ETH:\n    expected: {:?}\n    received: {:?}",
            expected_account_balance,
            account_balance
        );

        println!("  Smart account address: {deployed_account_address}");
        println!("  Using owner private key for revocation");
        println!(
            "  Session hash to revoke: 0x{}",
            hex::encode(session_hash.fixed_bytes())
        );

        // Revoke the initial session
        println!("Attempting to revoke session using owner's credentials...");

        let revoke_args = RevokeSessionArgs { session_id: session_hash };

        let signer = alloy::signers::local::PrivateKeySigner::from_str(
            owner_private_key,
        )?;
        let sign_fn = sign_fn_from_signer(signer);
        let revoke_result = revoke_session(
            revoke_args,
            deployed_account_address,
            sign_fn,
            &config,
        )
        .await?;

        println!("Session revocation successful:");
        println!(
            "  Transaction hash: {}",
            revoke_result.transaction_receipt.transaction_hash()
        );
        println!(
            "  Gas used: {:?}",
            revoke_result.transaction_receipt.gas_used()
        );
        println!("  Status: {:?}", revoke_result.transaction_receipt.status());

        eyre::ensure!(
            revoke_result.transaction_receipt.status(),
            "Revocation transaction should succeed"
        );

        // Step 6: Verify session is now revoked
        println!("\n--- Step 6: Verifying session is revoked ---");
        let revoked_session_state = get_session_state(
            GetSessionStateArgs {
                account: deployed_account_address,
                session_config: exact_session_config.clone(),
            },
            &config,
        )
        .await?;

        println!("Session state after revocation:");
        println!(
            "  Status: {:?} (2 = Closed/Revoked)",
            revoked_session_state.session_state.status
        );
        println!(
            "  Fees remaining: {:?}",
            revoked_session_state.session_state.fees_remaining
        );

        // Verify session is now closed/revoked (status = 2)
        eyre::ensure!(
            revoked_session_state.session_state.status.is_closed(),
            "Session should be closed/revoked (status=2)"
        );

        println!("✓ Session successfully revoked");

        // Step 7: Create a new session after revocation
        println!("\n--- Step 7: Creating a new session after revocation ---");

        // Create a second session configuration with different parameters for the transaction
        // Rich Wallet (3)
        let second_session_owner_private_key = owner_private_key; // Different key
        let second_session_owner_address =
            address!("90F79bf6EB2c4f870365E785982E1f101E93b906"); // ANOTHER ADDRESS NOT DERIVED FROM THE PRIVATE KEY ABOVE

        // Vitalik's address for the session transaction
        let vitalik_address =
            address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");

        let second_session_config = SessionSpec {
            signer: second_session_owner_address,
            expires_at: U256::from(1767225600u64), // January 1st, 2026 00:00:00 UTC
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(50000000000000000u64), // 0.05 ETH
                period: U256::from(0),
            },
            call_policies: vec![], // No call policies
            transfer_policies: vec![TransferSpec {
                target: vitalik_address, // Allow transfers to Vitalik's address
                max_value_per_use: U256::from(5000000000000000u64), // 0.005 ETH per transfer
                value_limit: UsageLimit {
                    limit_type: LimitType::Unlimited,
                    limit: U256::from(0),
                    period: U256::from(0),
                },
            }],
        };

        println!("Second session configuration:");
        println!("  Signer: {}", second_session_config.signer);
        println!("  Expires at: {}", second_session_config.expires_at);
        println!(
            "  Fee limit: {} wei (0.05 ETH)",
            second_session_config.fee_limit.limit
        );
        println!(
            "  Transfer max value per use: {} wei (0.005 ETH)",
            second_session_config.transfer_policies[0].max_value_per_use
        );

        // Create the session using the owner provider (simulating ECDSA client)
        let _second_session_key = {
            // Convert the hex string to FixedBytes<32>
            let private_key_bytes =
                hex::decode(second_session_owner_private_key)?;
            FixedBytes::<32>::from_slice(&private_key_bytes)
        };

        let create_session_args = CreateSessionArgs {
            account: deployed_account_address,
            session_config: second_session_config.clone(),
            paymaster: Some(PaymasterParams {
                paymaster: config.contracts.account_paymaster,
                paymaster_input: alloy::primitives::Bytes::new(),
            }),
        };

        println!("Creating second session using ECDSA-like client...");

        let second_session_signer =
            PrivateKeySigner::from_str(second_session_owner_private_key)?;
        let sign_fn = sign_fn_from_signer(second_session_signer);
        let second_session_result =
            create_session(create_session_args, sign_fn, &config).await?;

        println!("Second session created:");
        println!(
            "  Transaction hash: {}",
            second_session_result.transaction_receipt.transaction_hash()
        );
        println!(
            "  Status: {:?}",
            second_session_result.transaction_receipt.status()
        );

        eyre::ensure!(
            second_session_result.transaction_receipt.status(),
            "Second session creation should succeed"
        );

        // Check the status of the new session
        println!("\n--- Checking status of the new session ---");
        let second_session_hash =
            get_session_hash(second_session_config.clone())?;
        println!(
            "Second session hash: 0x{}",
            hex::encode(second_session_hash.fixed_bytes())
        );

        let second_session_state = get_session_state(
            GetSessionStateArgs {
                account: deployed_account_address,
                session_config: second_session_config.clone(),
            },
            &config,
        )
        .await?;

        println!("Second session state:");
        println!(
            "  Status: {:?} (1 = Active)",
            second_session_state.session_state.status
        );
        println!(
            "  Fees remaining: {:?}",
            second_session_state.session_state.fees_remaining
        );
        println!(
            "  Transfer value entries: {:?}",
            second_session_state.session_state.transfer_value.len()
        );

        // Verify the second session is active
        eyre::ensure!(
            second_session_state.session_state.status.is_active(),
            "Second session should be active"
        );
        eyre::ensure!(
            second_session_state.session_state.fees_remaining
                == second_session_config.fee_limit.limit,
            "Second session fee limit should match"
        );

        // // Send a transaction using SessionClient to Vitalik's address
        // println!("\n--- Sending session transaction using SessionClient ---");

        // let transfer_amount = U256::from(3000000000000000u64); // 0.003 ETH - within our 0.005 ETH limit

        // println!("Session transaction details:");
        // println!("  From (smart account): {}", deployed_account_address);
        // println!("  To (Vitalik): {}", vitalik_address);
        // println!("  Amount: {} wei (0.003 ETH)", transfer_amount);
        // println!(
        //     "  Session max per use: {} wei (0.005 ETH)",
        //     second_session_config.transferPolicies[0].maxValuePerUse
        // );
        // println!("  Session signer: {}", second_session_config.signer);

        // // Check balances before transfer
        // let balance_before_transfer =
        //     public_provider.get_balance(deployed_account_address).await?;
        // println!(
        //     "Smart account balance before transfer: {} wei",
        //     balance_before_transfer
        // );

        // let vitalik_balance_before =
        //     public_provider.get_balance(vitalik_address).await?;
        // println!(
        //     "Vitalik balance before transfer: {} wei",
        //     vitalik_balance_before
        // );

        // // Create SessionClient and send transaction
        // let session_client = SessionClient::new(
        //     deployed_account_address,
        //     second_session_key,
        //     second_session_config.clone(),
        //     config.clone(),
        // )?;

        // println!("SessionClient created:");
        // println!("  Smart account address: {}", deployed_account_address);
        // println!("  Session key: 0x{}", hex::encode(second_session_key));
        // println!(
        //     "  Session signer from config: {}",
        //     second_session_config.signer
        // );

        // // Create the transaction request for the transfer
        // let session_tx_request = TransactionRequest::default()
        //     .with_to(vitalik_address)
        //     .with_value(transfer_amount);

        // println!("Sending transaction through SessionClient...");
        // let session_transaction_receipt =
        //     session_client.send_transaction(session_tx_request).await?;

        // println!("Session transaction confirmed:");
        // println!("  Status: {:?}", session_transaction_receipt.status());
        // println!(
        //     "  Transaction hash: {}",
        //     session_transaction_receipt.transaction_hash()
        // );

        // eyre::ensure!(
        //     session_transaction_receipt.status(),
        //     "Session transaction should succeed"
        // );

        // // Check balances after transaction
        // let balance_after_transfer =
        //     public_provider.get_balance(deployed_account_address).await?;
        // println!(
        //     "Smart account balance after transfer: {} wei",
        //     balance_after_transfer
        // );

        // let vitalik_balance_after =
        //     public_provider.get_balance(vitalik_address).await?;
        // println!(
        //     "Vitalik balance after transfer: {} wei",
        //     vitalik_balance_after
        // );

        // // Verify the transfer happened
        // let transfer_difference =
        //     vitalik_balance_after - vitalik_balance_before;
        // println!("Transfer amount verified: {} wei", transfer_difference);
        // eyre::ensure!(
        //     transfer_difference == transfer_amount,
        //     "Transfer amount should match expected value"
        // );

        // // Check updated session state after transaction
        // println!("\n--- Checking updated session state after transaction ---");
        // let session_state_after_tx = get_session_state(
        //     GetSessionStateArgs {
        //         account: deployed_account_address,
        //         session_config: second_session_config.clone(),
        //     },
        //     &config,
        // )
        // .await?;

        // println!("Session state after transaction:");
        // println!(
        //     "  Status: {} (1 = Active)",
        //     session_state_after_tx.session_state.status
        // );
        // println!(
        //     "  Fees remaining: {}",
        //     session_state_after_tx.session_state.feesRemaining
        // );
        // println!(
        //     "  Transfer value entries: {}",
        //     session_state_after_tx.session_state.transferValue.len()
        // );

        // if !session_state_after_tx.session_state.transferValue.is_empty() {
        //     println!(
        //         "Transfer value remaining: {}",
        //         session_state_after_tx.session_state.transferValue[0].remaining
        //     );

        //     // The transfer value limit should have decreased by the transfer amount
        //     let original_transfer_limit_hex = "115792089237316195423570985008687907853269984665640564039457584007913129639935";
        //     let original_transfer_limit =
        //         U256::from_str(original_transfer_limit_hex)?;
        //     let expected_remaining = original_transfer_limit - transfer_amount;
        //     let actual_remaining =
        //         session_state_after_tx.session_state.transferValue[0].remaining;
        //     println!("Expected remaining: {}", expected_remaining);
        //     println!("Actual remaining: {}", actual_remaining);
        //     eyre::ensure!(
        //         actual_remaining == expected_remaining,
        //         "Transfer value limit should have decreased by the transfer amount"
        //     );
        // }

        // // Session should still be active after the transaction
        // eyre::ensure!(
        //     session_state_after_tx.session_state.status == 1,
        //     "Session should still be active after the transaction"
        // );

        // println!("Session transaction completed successfully! 🎉");

        // // println!("\n=== RUST SDK TEST COMPLETED SUCCESSFULLY (Steps 1-7) ===");
        // // println!("This test successfully:");
        // // println!("1. Deployed a smart account with initial session");
        // // println!("2. Verified session module is a validator");
        // // println!("3. Got initial session state and validated properties");
        // // println!("4. Calculated and verified session hash");
        // // println!("5. Funded smart account and revoked the first session");
        // // println!("6. Verified session is now closed/revoked");
        // // println!("7. Created a new session with different parameters");
        // // println!("All using SDK functions with config-based authentication.");

        drop(anvil_zksync);

        Ok(())
    }
}
