use crate::erc4337::{
    account::{
        erc7579::{IERC7579Account, module_installed::is_module_installed},
        modular_smart_account::send::send_transaction,
    },
    bundler::pimlico::client::BundlerClient,
    signer::Signer,
};
use alloy::{
    primitives::{Address, Bytes, U256},
    providers::Provider,
    sol_types::SolCall,
};

pub async fn add_module<P: Provider + Send + Sync + Clone>(
    account_address: Address,
    module_address: Address,
    entry_point_address: Address,
    provider: P,
    bundler_client: BundlerClient,
    signer: Signer,
) -> eyre::Result<()> {
    let module_type_id = 1;

    let init_data = Bytes::default();

    let call_data =
        add_module_call_data(module_address, module_type_id, init_data);

    send_transaction(
        account_address,
        entry_point_address,
        call_data,
        None,
        bundler_client,
        provider.clone(),
        signer,
    )
    .await?;

    let is_expected_module_installed =
        is_module_installed(module_address, account_address, provider.clone())
            .await?;

    eyre::ensure!(
        is_expected_module_installed,
        "{} is not installed",
        module_address
    );

    Ok(())
}

fn add_module_call_data(
    module: Address,
    module_type_id: u8,
    init_data: Bytes,
) -> Bytes {
    IERC7579Account::installModuleCall {
        moduleTypeId: U256::from(module_type_id),
        module,
        initData: init_data,
    }
    .abi_encode()
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        erc4337::account::modular_smart_account::{
            deploy::{EOASigners, deploy_account},
            signature::{eoa_signature, stub_signature_eoa},
        },
        utils::alloy_utilities::test_utilities::{
            TestInfraConfig,
            start_anvil_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::{
        primitives::{FixedBytes, U256, address},
        rpc::types::TransactionRequest,
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_add_module() -> eyre::Result<()> {
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

        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;

        let signers =
            vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let account_address = deploy_account(
            factory_address,
            Some(eoa_signers),
            None,
            provider.clone(),
        )
        .await?;

        println!("Account deployed");

        let is_eoa_module_installed = is_module_installed(
            eoa_validator_address,
            account_address,
            provider.clone(),
        )
        .await?;

        eyre::ensure!(
            is_eoa_module_installed,
            "is_eoa_module_installed is not installed"
        );

        {
            let fund_tx = TransactionRequest::default()
                .to(account_address)
                .value(U256::from(10000000000000000000u64));
            _ = provider.send_transaction(fund_tx).await?.get_receipt().await?;
        }
        let module_address = contracts.webauthn_validator;

        let stub_sig = stub_signature_eoa(eoa_validator_address)?;

        let signature_provider = {
            let signer_private_key = signer_private_key.clone();
            Arc::new(move |hash: FixedBytes<32>| {
                eoa_signature(&signer_private_key, eoa_validator_address, hash)
            })
        };

        let signer =
            Signer { provider: signature_provider, stub_signature: stub_sig };

        add_module(
            account_address,
            module_address,
            entry_point_address,
            provider.clone(),
            bundler_client.clone(),
            signer,
        )
        .await?;

        let is_web_authn_module_installed = is_module_installed(
            module_address,
            account_address,
            provider.clone(),
        )
        .await?;

        eyre::ensure!(
            is_web_authn_module_installed,
            "is_web_authn_module is not installed"
        );

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }
}
