use crate::erc4337::{
    account::modular_smart_account::{
        deploy::{
            DeployAccountParams, MSADeployAccount, create_init_data,
            generate_random_account_id, get_account_created_address,
        },
        send::{FactoryPayload, SendUserOpParams, send_user_op},
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

    let init_data =
        create_init_data(eoa_signers, webauthn_signer, session_validator);

    let factory_data: Bytes =
        MSADeployAccount::new(account_id, init_data).encode().into();

    let predicted_account_address = get_sender_address(
        &provider,
        factory_address,
        factory_data.clone(),
        entry_point_address,
    )
    .await?;

    let user_op_receipt = send_user_op(SendUserOpParams {
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
            account::modular_smart_account::deploy::EOASigners,
            paymaster::mock_paymaster::deploy_mock_paymaster_and_deposit_amount,
            signer::create_eoa_signer,
        },
        utils::alloy_utilities::test_utilities::{
            config::TestInfraConfig,
            start_node_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::{
        primitives::{U256, address},
        providers::WalletProvider,
    };

    #[tokio::test]
    async fn test_deploy_account_with_user_op() -> eyre::Result<()> {
        let (
            _,
            anvil_instance,
            provider,
            contracts,
            signer_private_key,
            bundler,
            bundler_client,
        ) = start_node_and_deploy_contracts_and_start_bundler_with_config(
            &TestInfraConfig::rich_wallet_9(),
        )
        .await?;

        let paymaster_fund_amount = U256::from(1_000_000_000_000_000_000u64);
        let signer_address = provider.default_signer_address();
        let signer_balance = provider.get_balance(signer_address).await?;
        eyre::ensure!(
            signer_balance > paymaster_fund_amount,
            "Signer wallet not funded: {signer_address} (balance {signer_balance})"
        );

        let entry_point_address = contracts.entry_point;

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
                paymaster_fund_amount,
                provider.clone(),
            )
            .await?;
        let paymaster =
            Some(PaymasterParams::default_paymaster(paymaster_address));

        let signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

        _ = deploy_account_with_user_op(DeployAccountWithUserOpParams {
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

        drop(mock_paymaster);
        drop(bundler);
        drop(anvil_instance);

        Ok(())
    }
}
