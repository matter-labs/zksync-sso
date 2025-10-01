use crate::erc4337::account::modular_smart_account::{
    IMSA::initializeAccountCall, MSAFactory, MSAFactory::deployAccountCall,
};
use alloy::{
    primitives::{Address, Bytes, FixedBytes},
    sol,
    sol_types::{SolCall, SolValue},
};
use alloy_provider::Provider;

pub struct MSAInitializeAccount(initializeAccountCall);

impl MSAInitializeAccount {
    pub fn new(modules: Vec<Address>, data: Vec<Bytes>) -> Self {
        Self(initializeAccountCall { modules, data })
    }

    pub fn encode(&self) -> Vec<u8> {
        initializeAccountCall::abi_encode(&self.0)
    }
}

sol! {
    struct SignersParams {
        address[] signers;
    }
}

pub struct MSADeployAccount(deployAccountCall);

impl MSADeployAccount {
    pub fn new(account_id: FixedBytes<32>, init_data: Bytes) -> Self {
        Self(deployAccountCall { accountId: account_id, initData: init_data })
    }

    pub fn encode(&self) -> Vec<u8> {
        deployAccountCall::abi_encode(&self.0)
    }
}

fn encode_signers_params(signers: Vec<Address>) -> Vec<u8> {
    SignersParams { signers: signers.to_vec() }.abi_encode_params()
}

pub struct EOASigners {
    pub addresses: Vec<Address>,
    pub validator_address: Address,
}

pub async fn deploy_account_basic<P: Provider + Send + Sync + Clone>(
    factory_address: Address,
    eoa_signers: Option<EOASigners>,
    provider: P,
) -> eyre::Result<()> {
    let factory = MSAFactory::new(factory_address, provider.clone());

    let account_id = FixedBytes::<32>::default();

    let (data, modules) = if let Some(signers) = eoa_signers {
        let eoa_signer_encoded =
            encode_signers_params(signers.addresses).into();
        let modules = vec![signers.validator_address];
        (vec![eoa_signer_encoded], modules)
    } else {
        (vec![], vec![])
    };

    let init_data: Bytes =
        MSAInitializeAccount::new(modules, data).encode().into();

    let deploy_account =
        factory.deployAccount(account_id, init_data).into_transaction_request();

    let _ =
        provider.send_transaction(deploy_account).await?.get_receipt().await?;

    Ok(())
}

pub fn deploy_account_via_user_op() -> eyre::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        primitives::address, providers::ProviderBuilder,
        signers::local::PrivateKeySigner,
    };
    use std::str::FromStr;

    #[tokio::test]
    async fn test_deploy_account_basic() -> eyre::Result<()> {
        let rpc_url = "http://localhost:8545".parse()?;

        let factory_address =
            address!("0x416C42991d05b31E9A6dC209e91AD22b79D87Ae6");

        let provider = {
            let signer_private_key = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
            let signer = PrivateKeySigner::from_str(&signer_private_key)?;
            let alloy_signer = signer.clone();
            let ethereum_wallet =
                alloy::network::EthereumWallet::new(alloy_signer.clone());

            let provider = ProviderBuilder::new()
                .wallet(ethereum_wallet.clone())
                .connect_http(rpc_url);

            provider
        };

        _ = deploy_account_basic(factory_address, None, provider.clone())
            .await?;

        Ok(())
    }

    #[test]
    fn test_deploy_account_via_user_op() -> eyre::Result<()> {
        Ok(())
    }
}
