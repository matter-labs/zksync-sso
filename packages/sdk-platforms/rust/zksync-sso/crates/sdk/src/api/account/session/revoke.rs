use crate::{
    client::session::actions::session::{
        revoke::{
            RevokeSessionArgs as SdkRevokeSessionArgs,
            revoke_session as client_revoke_session,
        },
        send::SignFn,
    },
    config::Config,
};
use alloy::{
    hex::{FromHex, FromHexError},
    primitives::{Address, FixedBytes},
    signers::local::PrivateKeySigner,
};
use alloy_zksync::wallet::ZksyncWallet;
use std::str::FromStr;

/// Arguments for revoking a session
#[derive(Debug, Clone)]
pub struct RevokeSessionArgs {
    /// Session ID hex string
    pub session_id: String,
}

/// Return type for session revocation
#[derive(Debug, Clone)]
pub struct RevokeSessionReturnType {
    /// Transaction receipt after session revocation
    pub transaction_receipt_json: String,
}

impl TryFrom<RevokeSessionArgs> for SdkRevokeSessionArgs {
    type Error = FromHexError;

    fn try_from(args: RevokeSessionArgs) -> Result<Self, Self::Error> {
        let session_id_bytes = FixedBytes::<32>::from_hex(args.session_id)?;
        Ok(SdkRevokeSessionArgs { session_id: session_id_bytes.into() })
    }
}

pub async fn revoke_session(
    args: RevokeSessionArgs,
    account_address: Address,
    sign_fn: SignFn,
    config: &Config,
) -> eyre::Result<RevokeSessionReturnType> {
    let args = args.try_into()?;
    let result =
        client_revoke_session(args, account_address, sign_fn, config).await?;
    let transaction_receipt_json =
        serde_json::to_string(&result.transaction_receipt).map_err(|e| {
            eyre::eyre!("Failed to serialize transaction receipt: {}", e)
        })?;
    Ok(RevokeSessionReturnType { transaction_receipt_json })
}

pub fn private_key_to_address(private_key_hex: &str) -> eyre::Result<Address> {
    let signer = PrivateKeySigner::from_str(private_key_hex)?;
    let wallet = ZksyncWallet::from(signer);
    let address = wallet.default_signer().address();
    Ok(address)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::account::fund::fund_account,
        client::{
            modular_account::is_module_validator,
            session::actions::session::{
                hash::get_session_hash,
                state::{GetSessionStateArgs, get_session_state},
            },
        },
        config::deploy_wallet::DeployWallet,
        utils::{
            anvil_zksync::rich_wallet::RichWallet,
            session::session_lib::session_spec::{
                SessionSpec, limit_type::LimitType,
                transfer_spec::TransferSpec, usage_limit::UsageLimit,
            },
            test_utils::{
                passkey::get_mock_credential_details,
                spawn_node_and_deploy_contracts,
            },
        },
    };
    use alloy::{
        primitives::{FixedBytes, U256, address, hex},
        providers::Provider,
    };
    use alloy_zksync::provider::zksync_provider;
    use url;

    #[tokio::test]
    async fn test_api_create_and_revoke_session() -> eyre::Result<()> {
        // Arrange
        let (anvil_zksync, mut config, _) =
            spawn_node_and_deploy_contracts().await?;

        let private_key = RichWallet::four().private_key_hex();

        // Owner private key for ECDSA smart account client
        let owner_private_key = "0x5de4111afa1a4b94908f83103c3a57e0c3c9e9da2dd5a02a84e9fde30d7e96c3";

        config.deploy_wallet =
            Some(DeployWallet { private_key_hex: private_key.to_string() });

        let owner_address = private_key_to_address(owner_private_key)?;
        println!("owner_address: {owner_address:?}");
        let expected_owner_address =
            address!("0x6a34Ea49c29BF7Cce95F51E7F0f419831Ad5dBC6");
        eyre::ensure!(
            owner_address == expected_owner_address,
            "owner_address does not match expected address, expected: {:?}, received: {:?}",
            expected_owner_address,
            owner_address
        );

        let transfer_session_target =
            address!("0xdeBbD4CE2Bd6BD869D3ac93666A0D5F4fc06FC72");
        let session_owner_address =
            address!("0x9BbC92a33F193174bf6Cc09c4b4055500d972479");
        let random_salt_str = "sdk-test-factory-replication-010";
        let random_salt =
            alloy::primitives::keccak256(random_salt_str.as_bytes()); // Unique ID for deterministic salt
        println!("random_salt: 0x{}", hex::encode(random_salt));
        let expires_at = 1749040108u64;

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

        // Step 1: Deploy account WITH initial session
        let deployed_account_address = {
            use crate::client::passkey::actions::deploy::{
                DeployAccountArgs, deploy_account,
            };
            use alloy::primitives::Bytes;
            use alloy_zksync::network::unsigned_tx::eip712::PaymasterParams;

            let args = {
                let deploy_account_credential = get_mock_credential_details();

                let unique_account_id = Some(random_salt_str.to_string());

                let contracts = config.clone().contracts;

                let origin: String = "https://example.com".to_string();

                let paymaster = Some(PaymasterParams {
                    paymaster: contracts.account_paymaster,
                    paymaster_input: Bytes::new(),
                });

                DeployAccountArgs {
                    credential: deploy_account_credential,
                    expected_origin: Some(origin),
                    unique_account_id,
                    paymaster,
                    contracts,
                    initial_k1_owners: Some(vec![owner_address]),
                    initial_session: Some(exact_session_config.clone()),
                }
            };

            let result = deploy_account(args, &config).await?;

            let deployed_account_address = result.address;

            println!(
                "XDB - test_deploy_account - Deployed account address: {deployed_account_address}"
            );

            deployed_account_address
        };

        println!("Account deployed successfully!");
        println!("  Deployed address: {deployed_account_address}");

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

        // Verify the session is active
        eyre::ensure!(
            initial_session_state.session_state.is_active(),
            "Initial session should be active (status=1)"
        );

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
            "Session hash does not match expected value, expected: {expected_session_hash:?}, received: {session_hash:?}",
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

        let session_hash_str = hex::encode(session_hash.fixed_bytes());

        let revoke_args = RevokeSessionArgs { session_id: session_hash_str };

        let signer = alloy::signers::local::PrivateKeySigner::from_str(
            owner_private_key,
        )?;
        let sign_fn =
            crate::client::session::actions::session::send::sign_fn_from_signer(
                signer,
            );
        let revoke_result = revoke_session(
            revoke_args,
            deployed_account_address,
            sign_fn,
            &config,
        )
        .await?;

        println!("Session revocation successful:");
        println!(
            "  Transaction receipt json: {}",
            revoke_result.transaction_receipt_json
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
            revoked_session_state.session_state.is_closed(),
            "Session should be closed/revoked (status=2)"
        );

        println!("✓ Session successfully revoked");

        drop(anvil_zksync);

        Ok(())
    }
}
