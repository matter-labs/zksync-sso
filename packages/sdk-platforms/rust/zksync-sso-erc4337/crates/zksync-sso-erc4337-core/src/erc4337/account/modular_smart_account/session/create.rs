use crate::erc4337::{
    account::{
        erc7579::calls::encoded_call_with_target_and_data,
        modular_smart_account::{
            send::{SendUserOpParams, send_user_op},
            session::{
                contract::SessionKeyValidator,
                session_lib::session_spec::SessionSpec,
            },
        },
    },
    bundler::pimlico::client::BundlerClient,
    paymaster::params::PaymasterParams,
    signer::Signer,
};
use alloy::{
    primitives::{Address, Bytes},
    providers::Provider,
    sol_types::SolCall,
};

#[derive(Clone)]
pub struct CreateSessionParams<P: Provider + Send + Sync + Clone> {
    pub account_address: Address,
    pub spec: SessionSpec,
    pub proof: Bytes,
    pub entry_point_address: Address,
    pub session_key_validator: Address,
    pub paymaster: Option<PaymasterParams>,
    pub bundler_client: BundlerClient,
    pub provider: P,
    pub signer: Signer,
}

pub async fn create_session<P>(
    params: CreateSessionParams<P>,
) -> eyre::Result<()>
where
    P: Provider + Send + Sync + Clone,
{
    let CreateSessionParams {
        account_address,
        spec,
        proof,
        entry_point_address,
        session_key_validator,
        paymaster,
        bundler_client,
        provider,
        signer,
    } = params;

    let call_data = add_session_call_data(spec, proof, session_key_validator);

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

pub fn create_session_call_data(spec: SessionSpec, proof: Bytes) -> Bytes {
    SessionKeyValidator::createSessionCall { sessionSpec: spec.into(), proof }
        .abi_encode()
        .into()
}

fn add_session_call_data(
    spec: SessionSpec,
    proof: Bytes,
    session_key_validator: Address,
) -> Bytes {
    let calldata = create_session_call_data(spec, proof);
    encoded_call_with_target_and_data(session_key_validator, calldata)
}

#[cfg(test)]
pub mod tests {
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
                        contract::SessionLib,
                        session_lib::session_spec::{
                            limit_type::LimitType, transfer_spec::TransferSpec,
                            usage_limit::UsageLimit,
                        },
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
    use alloy::{
        primitives::{
            Address, Bytes, U256, Uint, address, bytes, fixed_bytes, keccak256,
        },
        signers::{SignerSync, local::PrivateKeySigner},
        sol_types::SolValue,
    };
    use std::str::FromStr;

    pub fn generate_session_proof(
        session_spec: &SessionSpec,
        account_address: Address,
        signer_private_key: &str,
    ) -> eyre::Result<Bytes> {
        let session_lib_spec: SessionLib::SessionSpec =
            session_spec.clone().into();
        let session_hash = keccak256(session_lib_spec.abi_encode());
        let digest = keccak256((session_hash, account_address).abi_encode());

        let session_signer_instance =
            PrivateKeySigner::from_str(signer_private_key)?;
        Ok(session_signer_instance.sign_hash_sync(&digest)?.as_bytes().into())
    }

    #[tokio::test]
    async fn test_create_session() -> eyre::Result<()> {
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

        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        let session_key_module = contracts.session_validator;
        let eoa_validator_address = contracts.eoa_validator;
        let factory_address = contracts.account_factory;

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

        {
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

            // Calculate proof
            let proof = generate_session_proof(
                &session_spec,
                address,
                &signer_private_key,
            )?;

            create_session(CreateSessionParams {
                account_address: address,
                spec: session_spec,
                entry_point_address,
                session_key_validator: session_key_module,
                paymaster: None,
                bundler_client,
                provider,
                signer,
                proof,
            })
            .await?;
        }

        println!("Session successfully created");

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

        {
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

            // Calculate proof
            let proof = generate_session_proof(
                &session_spec,
                address,
                &signer_private_key,
            )?;

            create_session(CreateSessionParams {
                account_address: address,
                spec: session_spec,
                entry_point_address,
                session_key_validator: session_key_module,
                paymaster: None,
                bundler_client,
                provider,
                signer: signer.clone(),
                proof,
            })
            .await?;
        }

        println!("Session successfully created");

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }
}
