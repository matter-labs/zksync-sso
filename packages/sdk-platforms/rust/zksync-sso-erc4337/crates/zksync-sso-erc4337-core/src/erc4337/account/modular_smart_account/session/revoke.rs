use crate::erc4337::{
    account::{
        erc7579::{Execution, calls::encode_calls},
        modular_smart_account::{
            send::{SendParams, send_transaction},
            session::SessionKeyValidator,
        },
    },
    bundler::pimlico::client::BundlerClient,
    signer::Signer,
};
use alloy::{
    primitives::{Address, Bytes, FixedBytes, U256},
    providers::Provider,
    sol_types::SolCall,
};

pub async fn revoke_session<P: Provider + Send + Sync + Clone>(
    account_address: Address,
    session_hash: FixedBytes<32>,
    entry_point_address: Address,
    session_key_validator: Address,
    bundler_client: BundlerClient,
    provider: P,
    signer: Signer,
) -> eyre::Result<()> {
    let call_data =
        revoke_session_call_data(session_hash, session_key_validator);

    send_transaction(SendParams {
        account: account_address,
        entry_point: entry_point_address,
        call_data,
        nonce_key: None,
        paymaster: None,
        bundler_client,
        provider,
        signer,
    })
    .await?;

    Ok(())
}

fn revoke_session_call_data(
    session_hash: FixedBytes<32>,
    session_key_validator: Address,
) -> Bytes {
    let create_session_calldata =
        SessionKeyValidator::revokeKeyCall { sessionHash: session_hash }
            .abi_encode()
            .into();

    let call = {
        let target = session_key_validator;
        let value = U256::from(0);
        let data = create_session_calldata;
        Execution { target, value, data }
    };

    let calls = vec![call];
    encode_calls(calls).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        erc4337::{
            account::{
                erc7579::{
                    add_module::add_module,
                    module_installed::is_module_installed,
                },
                modular_smart_account::{
                    add_passkey::PasskeyPayload,
                    deploy::{
                        DeployAccountParams, EOASigners, WebAuthNSigner,
                        deploy_account,
                    },
                    send::passkey::tests::get_signature_from_js,
                    session::{
                        create::create_session,
                        hash::hash_session,
                        session_lib::session_spec::{
                            SessionSpec, limit_type::LimitType,
                            transfer_spec::TransferSpec,
                            usage_limit::UsageLimit,
                        },
                        status::get_session_status,
                    },
                    signature::{eoa_signature, stub_signature_eoa},
                    test_utilities::fund_account_with_default_amount,
                },
            },
            signer::Signer,
        },
        utils::alloy_utilities::test_utilities::{
            TestInfraConfig,
            start_anvil_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::primitives::{U256, Uint, address, bytes, fixed_bytes};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_revoke_session() -> eyre::Result<()> {
        let (
            _,
            anvil_instance,
            provider,
            contracts,
            signer_private_key,
            bundler,
            bundler_client,
        ) = {
            let signer_private_key = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6".to_string();
            let config = TestInfraConfig {
                signer_private_key: signer_private_key.clone(),
            };
            start_anvil_and_deploy_contracts_and_start_bundler_with_config(
                &config,
            )
            .await?
        };

        let session_key_module = contracts.session_validator;
        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;
        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        let signers =
            vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let address = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: Some(eoa_signers),
            webauthn_signer: None,
            id: None,
            provider: provider.clone(),
        })
        .await?;

        println!("Account deployed");

        let is_eoa_module_installed = is_module_installed(
            eoa_validator_address,
            address,
            provider.clone(),
        )
        .await?;

        eyre::ensure!(
            is_eoa_module_installed,
            "is_eoa_module_installed is not installed"
        );

        fund_account_with_default_amount(address, provider.clone()).await?;

        {
            let stub_sig = stub_signature_eoa(eoa_validator_address)?;
            let signer_private_key = signer_private_key.clone();
            let signature_provider = Arc::new(move |hash: FixedBytes<32>| {
                eoa_signature(&signer_private_key, eoa_validator_address, hash)
            });

            let signer = Signer {
                provider: signature_provider,
                stub_signature: stub_sig,
            };

            add_module(
                address,
                session_key_module,
                entry_point_address,
                provider.clone(),
                bundler_client.clone(),
                signer,
            )
            .await?;

            let is_session_key_module_installed = is_module_installed(
                session_key_module,
                address,
                provider.clone(),
            )
            .await?;

            eyre::ensure!(
                is_session_key_module_installed,
                "session_key_module is not installed"
            );

            println!("\n\n\nsession_key_module successfully installed\n\n\n")
        }

        let signer = {
            let stub_sig = stub_signature_eoa(eoa_validator_address)?;
            let signer_private_key = signer_private_key.clone();
            let signature_provider = Arc::new(move |hash: FixedBytes<32>| {
                eoa_signature(&signer_private_key, eoa_validator_address, hash)
            });
            Signer { provider: signature_provider, stub_signature: stub_sig }
        };

        let session_spec = {
            let signer_address =
                address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
            let expires_at = Uint::from(2088558400u64);
            let target = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
            let session_spec = SessionSpec {
                signer: signer_address,
                expires_at,
                call_policies: vec![],
                fee_limit: UsageLimit {
                    limit_type: LimitType::Lifetime,
                    limit: U256::from(1_000_000_000_000_000_000u64),
                    period: Uint::from(0),
                },
                transfer_policies: vec![TransferSpec {
                    max_value_per_use: U256::from(1),
                    target,
                    value_limit: UsageLimit {
                        limit_type: LimitType::Unlimited,
                        limit: U256::from(0),
                        period: Uint::from(0),
                    },
                }],
            };

            create_session(
                address,
                session_spec.clone(),
                entry_point_address,
                session_key_module,
                bundler_client.clone(),
                provider.clone(),
                signer.clone(),
            )
            .await?;

            session_spec.clone()
        };

        println!("\n\n\nSession successfully created\n\n\n");

        let session_hash = hash_session(session_spec.clone());

        let session_status = get_session_status(
            address,
            session_hash,
            session_key_module,
            provider.clone(),
        )
        .await?;

        println!("Session status: {:?}", session_status);

        let expected_session_status = 1;
        eyre::ensure!(
            session_status == expected_session_status,
            "Session is not active"
        );

        revoke_session(
            address,
            session_hash,
            entry_point_address,
            session_key_module,
            bundler_client,
            provider,
            signer,
        )
        .await?;

        println!("\n\n\nSession successfully revoked\n\n\n");

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_session_with_webauthn() -> eyre::Result<()> {
        let (
            _,
            anvil_instance,
            provider,
            contracts,
            _,
            bundler,
            bundler_client,
        ) = {
            let signer_private_key = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6".to_string();
            let config = TestInfraConfig {
                signer_private_key: signer_private_key.clone(),
            };
            start_anvil_and_deploy_contracts_and_start_bundler_with_config(
                &config,
            )
            .await?
        };

        let session_key_module = contracts.session_validator;
        let factory_address = contracts.account_factory;
        let webauthn_validator_address = contracts.webauthn_validator;
        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        let credential_id = bytes!("0x2868baa08431052f6c7541392a458f64");
        let passkey = [
            fixed_bytes!(
                "0xe0a43b9c64a2357ea7f66a0551f57442fbd32031162d9be762800864168fae40"
            ),
            fixed_bytes!(
                "0x450875e2c28222e81eb25ae58d095a3e7ca295faa3fc26fb0e558a0b571da501"
            ),
        ];
        let origin_domain = "https://example.com".to_string();
        let passkey = PasskeyPayload { credential_id, passkey, origin_domain };

        let passkey_signer = WebAuthNSigner {
            passkey,
            validator_address: webauthn_validator_address,
        };

        let address = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: None,
            webauthn_signer: Some(passkey_signer),
            id: None,
            provider: provider.clone(),
        })
        .await?;

        println!("Account deployed");

        let is_passkey_module_installed = is_module_installed(
            webauthn_validator_address,
            address,
            provider.clone(),
        )
        .await?;

        eyre::ensure!(
            is_passkey_module_installed,
            "is_passkey_module_installed is not installed"
        );

        fund_account_with_default_amount(address, provider.clone()).await?;

        let signature_provider = Arc::new(move |hash: FixedBytes<32>| {
            let result = get_signature_from_js(hash.to_string())?;
            Ok(result)
        });

        let signer = Signer {
            stub_signature: get_signature_from_js(
                FixedBytes::<32>::default().to_string(),
            )?,
            provider: signature_provider,
        };

        {
            add_module(
                address,
                session_key_module,
                entry_point_address,
                provider.clone(),
                bundler_client.clone(),
                signer.clone(),
            )
            .await?;

            let is_session_key_module_installed = is_module_installed(
                session_key_module,
                address,
                provider.clone(),
            )
            .await?;

            eyre::ensure!(
                is_session_key_module_installed,
                "session_key_module is not installed"
            );

            println!("\n\n\nsession_key_module successfully installed\n\n\n")
        }

        let signer_address =
            address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
        let expires_at = Uint::from(2088558400u64);
        let target = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
        let session_spec = SessionSpec {
            signer: signer_address,
            expires_at,
            call_policies: vec![],
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(1_000_000_000_000_000_000u64),
                period: Uint::from(0),
            },
            transfer_policies: vec![TransferSpec {
                max_value_per_use: U256::from(1),
                target,
                value_limit: UsageLimit {
                    limit_type: LimitType::Unlimited,
                    limit: U256::from(0),
                    period: Uint::from(0),
                },
            }],
        };

        create_session(
            address,
            session_spec.clone(),
            entry_point_address,
            session_key_module,
            bundler_client.clone(),
            provider.clone(),
            signer.clone(),
        )
        .await?;

        println!("\n\n\nSession successfully created\n\n\n");

        let session_hash = hash_session(session_spec.clone());

        let session_status = get_session_status(
            address,
            session_hash,
            session_key_module,
            provider.clone(),
        )
        .await?;

        println!("Session status: {:?}", session_status);

        let expected_session_status = 1;
        eyre::ensure!(
            session_status == expected_session_status,
            "Session is not active"
        );

        {
            let session_hash = hash_session(session_spec.clone());

            revoke_session(
                address,
                session_hash,
                entry_point_address,
                session_key_module,
                bundler_client,
                provider,
                signer,
            )
            .await?;
        }

        println!("\n\n\nSession successfully revoked\n\n\n");

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }
}
