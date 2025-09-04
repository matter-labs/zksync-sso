use crate::{
    client::session::actions::session::{
        hash::SessionHash,
        send::{SessionSend, SessionSendImpl, SignFn},
    },
    config::Config,
    contracts::SessionKeyValidator,
    utils::alloy::extensions::ProviderExt,
};
use alloy::primitives::Address;
use alloy_zksync::{
    network::receipt_response::ReceiptResponse as ZKReceiptResponse,
    provider::zksync_provider,
};
use log::debug;
use std::fmt::Debug;
use url;

/// Arguments for revoking a session
#[derive(Debug, Clone)]
pub struct RevokeSessionArgs {
    /// Session ID
    pub session_id: SessionHash,
}

/// Return type for session creation
#[derive(Debug, Clone)]
pub struct RevokeSessionReturnType {
    /// Transaction receipt after session creation
    pub transaction_receipt: ZKReceiptResponse,
}

pub async fn revoke_session(
    args: RevokeSessionArgs,
    account_address: Address,
    sign_fn: SignFn,
    config: &Config,
) -> eyre::Result<RevokeSessionReturnType> {
    let send_impl = SessionSendImpl::new(config, sign_fn, &account_address);
    revoke_session_with_send_fn(args, config, &send_impl).await
}

pub async fn revoke_session_with_send_fn(
    args: RevokeSessionArgs,
    config: &Config,
    send_fn: &impl SessionSend,
) -> eyre::Result<RevokeSessionReturnType> {
    let provider = {
        let node_url: url::Url = config.clone().node_url;

        zksync_provider().with_recommended_fillers().on_http(node_url.clone())
    };

    let session_validator =
        SessionKeyValidator::new(config.contracts.session, &provider);

    let session_hash = args.session_id;

    let call_builder = session_validator.revokeKey(session_hash.fixed_bytes());

    let transaction_request = call_builder.into_transaction_request();

    let pending_tx = send_fn.send(transaction_request).await?;
    debug!("Pending tx: {pending_tx:?}");

    let tx_hash = pending_tx.tx_hash().to_owned();
    debug!("Tx hash: {tx_hash:?}");

    let transaction_receipt =
        provider.wait_for_transaction_receipt(tx_hash).await?;
    debug!("Receipt: {transaction_receipt:?}");

    Ok(RevokeSessionReturnType { transaction_receipt })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::account::fund::fund_account,
        client::{
            modular_account::{
                DeployModularAccountArgs, SessionModuleArgs,
                deploy_modular_account, is_module_validator,
            },
            session::actions::session::{
                hash::get_session_hash,
                send::sign_fn_from_signer,
                state::{GetSessionStateArgs, get_session_state},
            },
        },
        config::deploy_wallet::DeployWallet,
        utils::{
            anvil_zksync::rich_wallet::RichWallet,
            session::session_lib::{
                session_spec::{
                    SessionSpec, limit_type::LimitType,
                    transfer_spec::TransferSpec, usage_limit::UsageLimit,
                },
                session_state::status::Status,
            },
            test_utils::{
                spawn_node_and_deploy_contracts,
                zksync_wallet_from_anvil_zksync,
            },
        },
    };
    use alloy::{
        network::ReceiptResponse,
        primitives::{FixedBytes, U256, address, hex, keccak256},
        providers::Provider,
        signers::local::PrivateKeySigner,
    };
    use alloy_zksync::{provider::zksync_provider, wallet::ZksyncWallet};
    use std::str::FromStr;
    use url;

    #[tokio::test]
    async fn test_create_and_revoke_session() -> eyre::Result<()> {
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

        let private_key = RichWallet::four().private_key_hex();

        // Owner private key for ECDSA smart account client
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
        let random_salt_str = "sdk-test-factory-replication-010";
        let random_salt = keccak256(random_salt_str.as_bytes()); // Unique ID for deterministic salt
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
                owners: vec![owner_address], // Use the ECDSA owner
                install_no_data_modules: vec![],
                session_module: Some(SessionModuleArgs {
                    location: config.contracts.session,
                    initial_session: Some(exact_session_config.clone()),
                }),
                paymaster: None,
                passkey_module: None,
                unique_account_id: Some(random_salt_str.to_string()),
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
            return Err(eyre::eyre!("Deployment transaction failed"));
        }

        // Step 2: Verify session module is a validator
        println!("\n--- Step 2: Verifying session module is a validator ---");

        let is_module_validator = is_module_validator(
            deployed_account_address,
            config.contracts.session,
            &config,
        )
        .await?;

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
            "  Transfer value entries: {}",
            initial_session_state.session_state.transfer_value.len()
        );
        println!(
            "  Call value entries: {}",
            initial_session_state.session_state.call_value.len()
        );
        println!(
            "  Call params entries: {}",
            initial_session_state.session_state.call_params.len()
        );

        // Verify the session is active
        eyre::ensure!(
            initial_session_state.session_state.is_active(),
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
            session_hash == expected_session_hash.into(),
            "Session hash does not match expected value"
        );

        // Verify session hash is deterministic and not empty
        let empty_hash = FixedBytes::<32>::from([0u8; 32]);
        eyre::ensure!(
            session_hash != empty_hash.into(),
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
        fund_account(deployed_account_address, funding_amount, &config).await?;

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

        let signer = PrivateKeySigner::from_str(owner_private_key)?;
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
            revoked_session_state.session_state.status == Status::Closed,
            "Session should be closed/revoked (status=2)"
        );

        println!("✓ Session successfully revoked");

        drop(anvil_zksync);

        Ok(())
    }
}
