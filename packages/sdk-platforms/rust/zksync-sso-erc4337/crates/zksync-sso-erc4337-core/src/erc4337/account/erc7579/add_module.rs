use crate::erc4337::account::erc7579::account::IERC7579Account;
use crate::erc4337::{
    account::{
        erc7579::{account::Execution, calls::encode_calls},
        modular_smart_account::signature::{eoa_signature, stub_signature_eoa},
    },
    bundler::models::estimate::Estimate,
    bundler::pimlico::client::BundlerClient,
    entry_point::EntryPoint,
    user_operation::hash::v08::get_user_operation_hash_entry_point,
};
use alloy::primitives::{Address, Bytes, U256};
use alloy::rpc::types::TransactionReceipt;
use alloy::{
    sol,
    sol_types::{SolCall, SolEvent, SolValue},
};
use alloy_provider::Provider;

pub fn add_module_call_data(
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
    use crate::erc4337::account::modular_smart_account::deploy::{
        EOASigners, deploy_account_basic, is_module_installed,
    };
    use crate::erc4337::account::modular_smart_account::send::send_transaction_eoa;
    use alloy::{
        primitives::{Bytes, U256, address},
        providers::ProviderBuilder,
        rpc::types::TransactionRequest,
        signers::local::PrivateKeySigner,
    };
    use std::str::FromStr;

    #[tokio::test]
    async fn test_add_module() -> eyre::Result<()> {
        let rpc_url = "http://localhost:8545".parse()?;

        let factory_address =
            address!("0x679FFF51F11C3f6CaC9F2243f9D14Cb1255F65A3");

        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        let eoa_validator_address =
            address!("0x00427eDF0c3c3bd42188ab4C907759942Abebd93");

        let signer_private_key = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";

        let provider = {
            let signer = PrivateKeySigner::from_str(&signer_private_key)?;
            let alloy_signer = signer.clone();
            let ethereum_wallet =
                alloy::network::EthereumWallet::new(alloy_signer.clone());

            let provider = ProviderBuilder::new()
                .wallet(ethereum_wallet.clone())
                .connect_http(rpc_url);

            provider
        };

        let signers =
            vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let address = deploy_account_basic(
            factory_address,
            Some(eoa_signers),
            provider.clone(),
        )
        .await?;

        println!("Account deployed");
        // let address = address!("0x6bf1c0c174e11b933e7d8940afadf8bb7b8d421c");

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

        let webauthn_module =
            address!("0xF3F924c9bADF6891D3676cfe9bF72e2C78527E17");
        let module_type_id = 1;
        let init_data = Bytes::default();
        let call_data =
            add_module_call_data(webauthn_module, module_type_id, init_data);

        let bundler_client = {
            use crate::erc4337::bundler::config::BundlerConfig;
            let bundler_url = "http://localhost:4337".to_string();
            let config = BundlerConfig::new(bundler_url);
            BundlerClient::new(config)
        };

        let _ = send_transaction_eoa(
            address,
            eoa_validator_address,
            entry_point_address,
            call_data,
            bundler_client,
            provider.clone(),
            signer_private_key.to_string(),
        )
        .await?;

        let is_web_authn_module_installed =
            is_module_installed(webauthn_module, address, provider.clone())
                .await?;

        eyre::ensure!(
            is_web_authn_module_installed,
            "is_web_authn_module is not installed"
        );

        Ok(())
    }
}
