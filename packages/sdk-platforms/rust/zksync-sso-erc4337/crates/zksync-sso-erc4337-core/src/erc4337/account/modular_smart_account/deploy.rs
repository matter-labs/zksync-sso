use crate::erc4337::account::modular_smart_account::{
    IMSA, IMSA::initializeAccountCall, MSAFactory,
    MSAFactory::deployAccountCall,
};
use alloy::{
    network::ReceiptResponse,
    primitives::{Address, Bytes, FixedBytes, U256},
    providers::Provider,
    rpc::types::TransactionReceipt,
    sol,
    sol_types::{SolCall, SolEvent, SolValue},
};
use eyre::{self, Ok};

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
) -> eyre::Result<Address> {
    let factory = MSAFactory::new(factory_address, provider.clone());

    let random_id = {
        use rand::{Rng, rng};

        pub fn generate_random_id() -> [u8; 32] {
            let mut random_bytes = [0u8; 32];
            rng().fill(&mut random_bytes);
            random_bytes
        }

        generate_random_id()
    };

    let account_id = FixedBytes::<32>::from_slice(&random_id);

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

    let deploy_account = factory.deployAccount(account_id, init_data);

    let receipt = deploy_account.send().await?.get_receipt().await?;

    let address = get_account_created_address(&receipt)?;

    Ok(address)
}

pub fn get_account_created_address(
    receipt: &TransactionReceipt,
) -> eyre::Result<Address> {
    let topic = MSAFactory::AccountCreated::SIGNATURE_HASH;
    let log = receipt
        .logs()
        .iter()
        .find(|log: &&alloy::rpc::types::Log| log.inner.topics()[0] == topic)
        .ok_or_else(|| eyre::eyre!("AccountCreated event not found in logs"))?;
    let event = log.inner.topics()[1];
    let address = Address::from_slice(&event[12..]);
    Ok(address)
}
pub fn deploy_account_via_user_op() -> eyre::Result<()> {
    Ok(())
}

pub async fn is_module_installed<P: Provider + Send + Sync + Clone>(
    module: Address,
    account: Address,
    provider: P,
) -> eyre::Result<bool> {
    let account = IMSA::new(account, provider.clone());
    let module_type_id = U256::from(1);
    let is_installed = account
        .isModuleInstalled(module_type_id, module, Bytes::default())
        .call()
        .await?;
    Ok(is_installed)
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
        // == Logs ==
        //   EOAKeyValidator: 0x00427eDF0c3c3bd42188ab4C907759942Abebd93
        //   SessionKeyValidator: 0x57eaa1Fd8d80135Db195B147a249aad777aD10f0
        //   WebAuthnValidator: 0xF3F924c9bADF6891D3676cfe9bF72e2C78527E17
        //   GuardianExecutor: 0x374ce0d25B00B909417d695237d06abFe4548eB1
        //   ModularSmartAccount implementation: 0x5646c10bFa3fA97B72402D26Bc66fEc0dbAf99c8
        //   UpgradeableBeacon: 0x7b1255B5DaBbBf84ADC423B8b6Ecd89F822A2f72
        //   MSAFactory: 0x679FFF51F11C3f6CaC9F2243f9D14Cb1255F65A3
        //   Initialized account: 0x6bf1C0c174e11B933e7d8940aFADf8BB7B8d421C

        let rpc_url = "http://localhost:8545".parse()?;

        let factory_address =
            address!("0x679FFF51F11C3f6CaC9F2243f9D14Cb1255F65A3");

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

        let _address =
            deploy_account_basic(factory_address, None, provider.clone())
                .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_deploy_account_with_eoa_signer() -> eyre::Result<()> {
        let rpc_url = "http://localhost:8545".parse()?;

        let factory_address =
            address!("0x679FFF51F11C3f6CaC9F2243f9D14Cb1255F65A3");

        let eoa_validator_address =
            address!("0x00427eDF0c3c3bd42188ab4C907759942Abebd93");

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

        let is_module_installed = is_module_installed(
            eoa_validator_address,
            address,
            provider.clone(),
        )
        .await?;
        eyre::ensure!(is_module_installed, "Module is not installed");

        Ok(())
    }

    #[test]
    fn test_deploy_account_via_user_op() -> eyre::Result<()> {
        Ok(())
    }
}
