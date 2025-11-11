use crate::erc4337::{
    account::modular_smart_account::{
        deploy::{
            DeployAccountParams, MSADeployAccount, create_init_data,
            generate_random_account_id, get_account_created_address,
        },
        send::{FactoryPayload, SendParams, send_transaction},
    },
    bundler::pimlico::client::BundlerClient,
    entry_point::sender_address::get_sender_address,
    paymaster::params::PaymasterParams,
    signer::Signer,
    utils::check_deployed::{Contract, check_contract_deployed},
};
use alloy::{
    primitives::{Address, Bytes, TxHash, Uint},
    providers::Provider,
};
use eyre::eyre;
use std::str::FromStr;

#[derive(Clone)]
pub struct DeployAccountWithUserOpParams<P: Provider + Send + Sync + Clone> {
    pub deploy_params: DeployAccountParams<P>,
    pub entry_point_address: Address,
    pub bundler_client: BundlerClient,
    pub signer: Signer,
    pub paymaster: Option<PaymasterParams>,
    pub nonce_key: Option<Uint<192, 3>>,
}

pub async fn deploy_account_with_user_op<P>(
    params: DeployAccountWithUserOpParams<P>,
) -> eyre::Result<Address>
where
    P: Provider + Send + Sync + Clone,
{
    let DeployAccountWithUserOpParams {
        deploy_params,
        entry_point_address,
        bundler_client,
        signer,
        paymaster,
        nonce_key,
    } = params;

    let DeployAccountParams {
        factory_address,
        eoa_signers,
        webauthn_signer,
        session_validator,
        id,
        provider,
    } = deploy_params;

    let account_id = id.unwrap_or(generate_random_account_id());

    let init_data = create_init_data(eoa_signers, webauthn_signer, session_validator);

    let factory_data: Bytes =
        MSADeployAccount::new(account_id, init_data).encode().into();

    let predicted_account_address = get_sender_address(
        &provider,
        factory_address,
        factory_data.clone(),
        entry_point_address,
    )
    .await?;

    let user_op_receipt = send_transaction(SendParams {
        account: predicted_account_address,
        entry_point: entry_point_address,
        factory_payload: Some(FactoryPayload {
            factory: factory_address,
            factory_data: Some(factory_data.clone()),
        }),
        call_data: Bytes::default(),
        nonce_key,
        paymaster,
        bundler_client,
        provider: provider.clone(),
        signer,
    })
    .await?;

    let tx_hash = TxHash::from_str(&user_op_receipt.receipt.transaction_hash)
        .map_err(|err| eyre!("Invalid transaction hash: {err}"))?;

    let receipt = provider
        .clone()
        .get_transaction_receipt(tx_hash)
        .await?
        .ok_or_else(|| eyre!("Transaction receipt not found"))?;

    let address = get_account_created_address(&receipt)?;

    if address != predicted_account_address {
        return Err(eyre!(
            "Deployed account address mismatch. Expected {predicted_account_address}, got {address}"
        ));
    }

    check_contract_deployed(
        &Contract { address, name: "ModularSmartAccount".to_string() },
        provider,
    )
    .await?;

    Ok(address)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        erc4337::{
            account::{
                erc7579::module_installed::is_module_installed,
                modular_smart_account::{
                    deploy::EOASigners,
                    test_utilities::fund_account_with_default_amount,
                },
            },
            paymaster::mock_paymaster::deploy_mock_paymaster_and_deposit_amount,
            signer::create_eoa_signer,
        },
        utils::alloy_utilities::test_utilities::{
            TestInfraConfig,
            start_anvil_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::primitives::{U256, address};

    #[tokio::test]
    async fn test_deploy_account_with_user_op_basic() -> eyre::Result<()> {
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
            start_anvil_and_deploy_contracts_and_start_bundler_with_config(
                &TestInfraConfig {
                    signer_private_key: signer_private_key.clone(),
                },
            )
            .await?
        };

        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;

        let signers =
            vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let (mock_paymaster, paymaster_address) =
            deploy_mock_paymaster_and_deposit_amount(
                U256::from(1_000_000_000_000_000_000u64),
                provider.clone(),
            )
            .await?;
        let paymaster =
            Some(PaymasterParams::default_paymaster(paymaster_address));

        let signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

        let address =
            deploy_account_with_user_op(DeployAccountWithUserOpParams {
                deploy_params: DeployAccountParams {
                    factory_address,
                    eoa_signers: Some(eoa_signers),
                    webauthn_signer: None,
                    session_validator: None,
                    id: None,
                    provider: provider.clone(),
                },
                entry_point_address,
                bundler_client: bundler_client.clone(),
                signer,
                paymaster,
                nonce_key: None,
            })
            .await?;

        fund_account_with_default_amount(address, provider.clone()).await?;

        let is_module_installed = is_module_installed(
            eoa_validator_address,
            address,
            provider.clone(),
        )
        .await?;
        eyre::ensure!(is_module_installed, "Module is not installed");

        drop(mock_paymaster);
        drop(bundler);
        drop(anvil_instance);

        Ok(())
    }
}
