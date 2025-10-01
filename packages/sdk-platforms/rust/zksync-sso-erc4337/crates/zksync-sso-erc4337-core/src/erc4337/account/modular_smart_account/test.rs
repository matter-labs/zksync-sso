use crate::erc4337::account::modular_smart_account::{
    IMSA, IMSA::initializeAccountCall, MSAFactory, MSAFactory::AccountCreated,
    MSAFactory::deployAccountCall,
};
use alloy::network::ReceiptResponse;
use alloy::sol_types::SolEvent;
use alloy::{
    primitives::{Address, Bytes, FixedBytes, U256, address},
    rpc::types::TransactionReceipt,
    sol,
    sol_types::{SolCall, SolValue},
};
use alloy_provider::Provider;
use eyre;
use eyre::Ok;

// == Logs ==
//   EOAKeyValidator: 0x34A1D3fff3958843C43aD80F30b94c510645C316
//   SessionKeyValidator: 0xA8452Ec99ce0C64f20701dB7dD3abDb607c00496
//   WebAuthnValidator: 0xDB8cFf278adCCF9E9b5da745B44E754fC4EE3C76
//   GuardianExecutor: 0x62c20Aa1e0272312BC100b4e23B4DC1Ed96dD7D1
//   ModularSmartAccount implementation: 0xDEb1E9a6Be7Baf84208BB6E10aC9F9bbE1D70809
//   UpgradeableBeacon: 0xD718d5A27a29FF1cD22403426084bA0d479869a0
//   MSAFactory: 0x416C42991d05b31E9A6dC209e91AD22b79D87Ae6

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
    println!("topic: {}", topic);
    dbg!(receipt.logs());
    let log = receipt
        .logs()
        .iter()
        .find(|log: &&alloy::rpc::types::Log| log.inner.topics()[0] == topic)
        .ok_or_else(|| eyre::eyre!("AccountCreated event not found in logs"))?;
    let event = log.inner.topics()[1].clone();
    let address = Address::from_slice(&event[12..]);
    Ok(address)
}
pub fn deploy_account_via_user_op() -> eyre::Result<()> {
    Ok(())
}

pub async fn is_module_installed<P: Provider + Send + Sync + Clone>(
    module: Address,
    provider: P,
) -> eyre::Result<bool> {
    let account = IMSA::new(module, provider.clone());
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
            address!("0x416C42991d05b31E9A6dC209e91AD22b79D87Ae6");

        let eoa_validator_address =
            address!("0x34A1D3fff3958843C43aD80F30b94c510645C316");

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

        _ = deploy_account_basic(
            factory_address,
            Some(eoa_signers),
            provider.clone(),
        )
        .await?;

        // is_module_installed()

        Ok(())
    }

    #[test]
    fn test_deploy_account_via_user_op() -> eyre::Result<()> {
        Ok(())
    }
}
