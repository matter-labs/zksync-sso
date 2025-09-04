#[cfg(test)]
mod tests {
    use crate::{
        client::session::actions::session::tests::get_session_key_payload,
        contracts::SessionKeyValidator,
        utils::{
            session::session_lib::session_spec::{
                SessionSpec, call_spec::CallSpec, limit_type::LimitType,
                transfer_spec::TransferSpec, usage_limit::UsageLimit,
            },
            test_utils::spawn_node_and_deploy_contracts,
        },
    };
    use alloy::{
        primitives::{Address, Bytes, U256, address, hex},
        signers::local::PrivateKeySigner,
    };
    use alloy_zksync::{provider::zksync_provider, wallet::ZksyncWallet};
    use eyre::{Ok, eyre};
    use url;

    #[derive(Default)]
    pub struct PartialSession {
        pub expires_at: Option<U256>,
        pub fee_limit: Option<UsageLimit>,
        pub call_policies: Option<Vec<CallSpec>>,
        pub transfer_policies: Option<Vec<TransferSpec>>,
    }

    /// Creates a SessionSpec with the given parameters, replicating the JavaScript SessionTester.getSession logic
    ///
    /// This function replicates the session creation logic from the JavaScript test to ensure
    /// compatibility and consistent encoding between Rust and JavaScript implementations.
    ///
    /// Parameters:
    /// - signer: The address of the session owner (equivalent to this.sessionOwner.address in JS)
    /// - partial_session: The partial session configuration with optional fields
    fn create_js_compatible_session(
        signer: Address,
        partial_session: PartialSession,
    ) -> SessionSpec {
        // From JS: session.expiresAt ?? Math.floor(Date.now() / 1000) + 60 * 60 * 24
        // For deterministic testing, we use a static timestamp if not provided
        let expires_at =
            partial_session.expires_at.unwrap_or(U256::from(1749040108u64));

        // From JS: session.feeLimit ? getLimit(session.feeLimit) : getLimit({ limit: parseEther("0.1") })
        let fee_limit = match partial_session.fee_limit {
            Some(limit) => limit,
            None => UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(100000000000000000u64), // parseEther("0.1")
                period: U256::from(0),
            },
        };

        // From JS: session.callPolicies?.map(...) ?? []
        let call_policies = partial_session.call_policies.unwrap_or_default();

        // From JS: session.transferPolicies?.map(...) ?? []
        let transfer_policies =
            partial_session.transfer_policies.unwrap_or_default();

        SessionSpec {
            signer,
            expires_at,
            fee_limit,
            call_policies,
            transfer_policies,
        }
    }

    /// `const initSessionData = abiCoder.encode(sessionKeyModuleContract.interface.getFunction("createSession").inputs, [initialSession]);`
    #[allow(clippy::type_complexity)]
    fn get_init_session_data(
        session_key_module_contract: &SessionKeyValidator::SessionKeyValidatorInstance<(), &alloy::providers::fillers::FillProvider<alloy::providers::fillers::JoinFill<alloy::providers::fillers::JoinFill<alloy::providers::Identity, alloy::providers::fillers::JoinFill<alloy_zksync::provider::fillers::Eip712FeeFiller, alloy::providers::fillers::JoinFill<alloy::providers::fillers::NonceFiller, alloy::providers::fillers::ChainIdFiller>>>, alloy::providers::fillers::WalletFiller<ZksyncWallet>>, alloy::providers::RootProvider<alloy_zksync::network::Zksync>, alloy_zksync::network::Zksync>, alloy_zksync::network::Zksync>,
        initial_session: &SessionSpec,
    ) -> Bytes {
        let init_session_data_with_selector = session_key_module_contract
            .createSession(initial_session.clone().into())
            .calldata()
            .to_owned();

        // Extract just the parameters (without the 4-byte function selector)
        // The JavaScript test uses abiCoder.encode(...inputs, [initialSession]) which doesn't include the selector
        if init_session_data_with_selector.len() >= 4 {
            Bytes::from(init_session_data_with_selector[4..].to_vec())
        } else {
            init_session_data_with_selector
        }
    }

    #[tokio::test]
    async fn test_session_encoding_matches_js() -> eyre::Result<()> {
        // Expected init_session_data from JavaScript test
        let expected_init_session_data = Bytes::from(hex::decode(
            "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000009bbc92a33f193174bf6cc09c4b4055500d9724790000000000000000000000000000000000000000000000000000000068403bec0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000debbd4ce2bd6bd869d3ac93666a0d5f4fc06fc72000000000000000000000000000000000000000000000000002386f26fc10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        )?);

        // Set up minimal test environment - we only need the session key module contract for encoding
        let (_, config, _deploy_provider) =
            spawn_node_and_deploy_contracts().await?;

        let provider = {
            let node_url: url::Url = config.clone().node_url;
            let signer = PrivateKeySigner::random();
            let wallet = ZksyncWallet::from(signer);

            zksync_provider()
                .with_recommended_fillers()
                .wallet(wallet)
                .on_http(node_url)
        };

        let session_key_module_address = config.contracts.session;
        let session_key_module_contract =
            SessionKeyValidator::new(session_key_module_address, &provider);

        // Construct session with exact same parameters as JavaScript test
        let initial_session = create_js_compatible_session(
            address!("0x9BbC92a33F193174bf6Cc09c4b4055500d972479"),
            PartialSession {
                expires_at: Some(U256::from(1749040108u64)),
                fee_limit: Some(UsageLimit {
                    limit_type: LimitType::Lifetime,
                    limit: U256::from(100000000000000000u64),
                    period: U256::from(0),
                }),
                call_policies: None,
                transfer_policies: Some(vec![TransferSpec {
                    target: address!(
                        "0xdeBbD4CE2Bd6BD869D3ac93666A0D5F4fc06FC72"
                    ),
                    max_value_per_use: U256::from(10000000000000000u64),
                    value_limit: UsageLimit {
                        limit_type: LimitType::Unlimited,
                        limit: U256::from(0),
                        period: U256::from(0),
                    },
                }]),
            },
        );

        // Encode the session data using the contract's createSession function
        let init_session_data = get_init_session_data(
            &session_key_module_contract,
            &initial_session,
        );

        println!(
            "Expected init session data length: {}",
            expected_init_session_data.len()
        );
        println!(
            "Actual init session data length: {}",
            init_session_data.len()
        );
        println!(
            "Expected: 0x{}",
            alloy::hex::encode(&expected_init_session_data)
        );
        println!("Actual:   0x{}", alloy::hex::encode(&init_session_data));

        // Compare the encoded data
        if init_session_data == expected_init_session_data {
            println!("✅ Init session data matches JS test exactly!");
        } else {
            println!("❌ Init session data differs from JS test");

            // Find first difference for debugging
            let min_len = std::cmp::min(
                expected_init_session_data.len(),
                init_session_data.len(),
            );
            for i in 0..min_len {
                if expected_init_session_data[i] != init_session_data[i] {
                    println!(
                        "First difference at byte {}: expected=0x{:02x}, actual=0x{:02x}",
                        i, expected_init_session_data[i], init_session_data[i]
                    );
                    break;
                }
            }

            return Err(eyre!(
                "Session encoding does not match JavaScript expected value"
            ));
        }

        // Assert they match exactly
        eyre::ensure!(
            init_session_data == expected_init_session_data,
            "Encoded session data should match JavaScript expected value exactly"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_session_key_payload() -> eyre::Result<()> {
        // Input values from JS test
        let session_key_module_address =
            address!("0xdBb08317Af6f7180b7Dc94758e44e51050a84e58");
        let init_session_data = Bytes::from(hex::decode(
            "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000009bbc92a33f193174bf6cc09c4b4055500d9724790000000000000000000000000000000000000000000000000000000068403bec0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000debbd4ce2bd6bd869d3ac93666a0d5f4fc06fc72000000000000000000000000000000000000000000000000002386f26fc10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        )?);

        // Expected output from JS test
        let expected_session_key_payload = Bytes::from(hex::decode(
            "000000000000000000000000dbb08317af6f7180b7dc94758e44e51050a84e58000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000000200000000000000000000000009bbc92a33f193174bf6cc09c4b4055500d9724790000000000000000000000000000000000000000000000000000000068403bec0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000debbd4ce2bd6bd869d3ac93666a0d5f4fc06fc72000000000000000000000000000000000000000000000000002386f26fc10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        )?);

        // Call the function under test
        let actual_session_key_payload = get_session_key_payload(
            session_key_module_address,
            init_session_data,
        );

        // Assert the output matches the expected result
        eyre::ensure!(
            actual_session_key_payload == expected_session_key_payload,
            "Session key payload should match the expected JS output"
        );

        Ok(())
    }
}
