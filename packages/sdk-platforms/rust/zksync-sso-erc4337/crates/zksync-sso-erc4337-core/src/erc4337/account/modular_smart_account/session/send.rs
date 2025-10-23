use alloy::primitives::{Address, Uint};

pub fn keyed_nonce(session_signer_address: Address) -> Uint<192, 3> {
    let nonce_bytes = session_signer_address.as_slice();
    Uint::from_be_slice(nonce_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        erc4337::{
            account::{
                erc7579::{
                    Execution, add_module::add_module, calls::encode_calls,
                    module_installed::is_module_installed,
                },
                modular_smart_account::{
                    deploy::{EOASigners, deploy_account},
                    send::send_transaction,
                    session::{
                        SessionLib::{SessionSpec, TransferSpec, UsageLimit},
                        create::create_session,
                    },
                    signature::{
                        eoa_signature, session_signature, stub_signature_eoa,
                    },
                },
            },
            signer::Signer,
        },
        utils::alloy_utilities::test_utilities::{
            TestInfraConfig,
            start_anvil_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::{
        primitives::{Bytes, FixedBytes, U256, Uint, address},
        providers::Provider,
        rpc::types::TransactionRequest,
        signers::local::PrivateKeySigner,
    };
    use std::{str::FromStr, sync::Arc};

    #[tokio::test]
    async fn test_keyed_nonce() -> eyre::Result<()> {
        let expected_keyed_nonce = Uint::from_str_radix(
            "CEbb58e4082Af6FaC6Ea275740f10073d1610ad9",
            16,
        )?;

        let session_signer_address =
            address!("0xCEbb58e4082Af6FaC6Ea275740f10073d1610ad9");

        let keyed_nonce = keyed_nonce(session_signer_address);

        eyre::ensure!(
            expected_keyed_nonce == keyed_nonce,
            "Keyed nonce mismatch, expected: {}, got: {}",
            expected_keyed_nonce,
            keyed_nonce
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_send_transaction_session() -> eyre::Result<()> {
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

        let session_key_hex = "0xb1da23908ba44fb1c6147ac1b32a1dbc6e7704ba94ec495e588d1e3cdc7ca6f9";
        println!("\n\n\nsession_key_hex: {}", session_key_hex);
        let session_signer_address =
            PrivateKeySigner::from_str(session_key_hex)?.address();
        println!("session_key address: {}", session_signer_address);

        let signers =
            vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let address = deploy_account(
            factory_address,
            Some(eoa_signers),
            None,
            provider.clone(),
        )
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

        {
            let fund_tx = TransactionRequest::default()
                .to(address)
                .value(U256::from(10000000000000000000u64));
            _ = provider.send_transaction(fund_tx).await?.get_receipt().await?;
        }

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

        let expires_at = Uint::from(2088558400u64);
        let target = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
        let session_spec = SessionSpec {
            signer: session_signer_address,
            expiresAt: expires_at,
            callPolicies: vec![],
            feeLimit: UsageLimit {
                limitType: 1,
                limit: U256::from(1_000_000_000_000_000_000u64),
                period: Uint::from(0),
            },
            transferPolicies: vec![TransferSpec {
                maxValuePerUse: U256::from(1),
                target,
                valueLimit: UsageLimit {
                    limitType: 0,
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

        println!("Session successfully created");

        // Send transaction using session signer
        let call = {
            let value = U256::from(1);
            let data = Bytes::default();
            Execution { target, value, data }
        };

        let calls = vec![call];
        let calldata = encode_calls(calls).into();

        let session_signer = {
            let stub_sig = session_signature(
                session_key_hex,
                session_key_module,
                &session_spec,
                Default::default(),
            )?;
            let signature_provider = Arc::new(move |hash: FixedBytes<32>| {
                session_signature(
                    session_key_hex,
                    session_key_module,
                    &session_spec,
                    hash,
                )
            });
            Signer { provider: signature_provider, stub_signature: stub_sig }
        };

        let keyed_nonce = keyed_nonce(session_signer_address);

        send_transaction(
            address,
            entry_point_address,
            calldata,
            Some(keyed_nonce),
            bundler_client,
            provider.clone(),
            session_signer,
        )
        .await?;

        println!("Session transaction successfully sent");

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }
}
