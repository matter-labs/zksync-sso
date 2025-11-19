use crate::erc4337::{
    account::{
        erc7579::calls::encoded_call_with_target_and_data,
        modular_smart_account::{
            send::{SendUserOpParams, send_user_op},
            session::contract::SessionKeyValidator,
        },
    },
    bundler::pimlico::client::BundlerClient,
    paymaster::params::PaymasterParams,
    signer::Signer,
};
use alloy::{
    primitives::{Address, Bytes, FixedBytes},
    providers::Provider,
    sol_types::SolCall,
};

#[derive(Clone)]
pub struct RevokeSessionParams<P: Provider + Send + Sync + Clone> {
    pub account_address: Address,
    pub session_hash: FixedBytes<32>,
    pub entry_point_address: Address,
    pub session_key_validator: Address,
    pub paymaster: Option<PaymasterParams>,
    pub bundler_client: BundlerClient,
    pub provider: P,
    pub signer: Signer,
}

pub async fn revoke_session<P: Provider + Send + Sync + Clone>(
    params: RevokeSessionParams<P>,
) -> eyre::Result<()> {
    let RevokeSessionParams {
        account_address,
        session_hash,
        entry_point_address,
        session_key_validator,
        paymaster,
        bundler_client,
        provider,
        signer,
    } = params;

    let call_data =
        revoke_session_call_data(session_hash, session_key_validator);

    send_user_op(SendUserOpParams {
        account: account_address,
        entry_point: entry_point_address,
        factory_payload: None,
        call_data,
        nonce_key: None,
        paymaster,
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

    encoded_call_with_target_and_data(
        session_key_validator,
        create_session_calldata,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        erc4337::{
            account::{
                erc7579::module::{
                    Module,
                    add::{AddModuleParams, AddModulePayload, add_module},
                    installed::{IsModuleInstalledParams, is_module_installed},
                },
                modular_smart_account::{
                    deploy::{
                        DeployAccountParams, EOASigners, WebAuthNSigner,
                        deploy_account,
                    },
                    passkey::add::PasskeyPayload,
                    session::{
                        create::{CreateSessionParams, create_session},
                        hash::hash_session,
                        session_lib::session_spec::{
                            SessionSpec, limit_type::LimitType,
                            transfer_spec::TransferSpec,
                            usage_limit::UsageLimit,
                        },
                        status::get_session_status,
                    },
                    test_utilities::fund_account_with_default_amount,
                },
            },
            signer::{
                create_eoa_signer, test_utils::create_test_webauthn_js_signer,
            },
        },
        utils::alloy_utilities::test_utilities::{
            TestInfraConfig,
            start_anvil_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::primitives::{U256, Uint, address, bytes, fixed_bytes};

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
            session_validator: None,
        })
        .await?;

        println!("Account deployed");

        let is_eoa_module_installed =
            is_module_installed(IsModuleInstalledParams {
                module: Module::eoa_validator(eoa_validator_address),
                account: address,
                provider: provider.clone(),
            })
            .await?;

        eyre::ensure!(
            is_eoa_module_installed,
            "is_eoa_module_installed is not installed"
        );

        fund_account_with_default_amount(address, provider.clone()).await?;

        {
            let signer = create_eoa_signer(
                signer_private_key.clone(),
                eoa_validator_address,
            )?;

            add_module(AddModuleParams {
                account_address: address,
                module: AddModulePayload::session_key(session_key_module),
                entry_point_address,
                paymaster: None,
                provider: provider.clone(),
                bundler_client: bundler_client.clone(),
                signer,
            })
            .await?;

            let is_session_key_module_installed =
                is_module_installed(IsModuleInstalledParams {
                    module: Module::session_key_validator(session_key_module),
                    account: address,
                    provider: provider.clone(),
                })
                .await?;

            eyre::ensure!(
                is_session_key_module_installed,
                "session_key_module is not installed"
            );

            println!("\n\n\nsession_key_module successfully installed\n\n\n")
        }

        let signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

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

            create_session(CreateSessionParams {
                account_address: address,
                spec: session_spec.clone(),
                entry_point_address,
                session_key_validator: session_key_module,
                paymaster: None,
                bundler_client: bundler_client.clone(),
                provider: provider.clone(),
                signer: signer.clone(),
            })
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

        eyre::ensure!(session_status.is_active(), "Session is not active");

        revoke_session(RevokeSessionParams {
            account_address: address,
            session_hash,
            entry_point_address,
            session_key_validator: session_key_module,
            paymaster: None,
            bundler_client: bundler_client.clone(),
            provider: provider.clone(),
            signer: signer.clone(),
        })
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
            session_validator: None,
        })
        .await?;

        println!("Account deployed");

        let is_passkey_module_installed =
            is_module_installed(IsModuleInstalledParams {
                module: Module::webauthn_validator(webauthn_validator_address),
                account: address,
                provider: provider.clone(),
            })
            .await?;

        eyre::ensure!(
            is_passkey_module_installed,
            "is_passkey_module_installed is not installed"
        );

        fund_account_with_default_amount(address, provider.clone()).await?;

        let signer = create_test_webauthn_js_signer();

        {
            add_module(AddModuleParams {
                account_address: address,
                module: AddModulePayload::session_key(session_key_module),
                entry_point_address,
                paymaster: None,
                provider: provider.clone(),
                bundler_client: bundler_client.clone(),
                signer: signer.clone(),
            })
            .await?;

            let is_session_key_module_installed =
                is_module_installed(IsModuleInstalledParams {
                    module: Module::session_key_validator(session_key_module),
                    account: address,
                    provider: provider.clone(),
                })
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

        create_session(CreateSessionParams {
            account_address: address,
            spec: session_spec.clone(),
            entry_point_address,
            session_key_validator: session_key_module,
            paymaster: None,
            bundler_client: bundler_client.clone(),
            provider: provider.clone(),
            signer: signer.clone(),
        })
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

        eyre::ensure!(session_status.is_active(), "Session is not active");

        {
            let session_hash = hash_session(session_spec.clone());

            revoke_session(RevokeSessionParams {
                account_address: address,
                session_hash,
                entry_point_address,
                session_key_validator: session_key_module,
                paymaster: None,
                bundler_client: bundler_client.clone(),
                provider: provider.clone(),
                signer: signer.clone(),
            })
            .await?;
        }

        println!("\n\n\nSession successfully revoked\n\n\n");

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }
}
