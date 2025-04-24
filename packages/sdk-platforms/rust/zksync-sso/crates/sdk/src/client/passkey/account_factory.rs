use crate::{
    client::{
        contracts::AAFactory,
        passkey::actions::deploy::{
            CredentialDetails, DeployAccountArgs, deploy_account,
        },
    },
    config::Config,
    utils::contract_deployed::{Contract, check_contract_deployed},
};
use alloy::{
    primitives::{Address, Bytes, FixedBytes, keccak256},
    providers::Provider,
    signers::local::PrivateKeySigner,
};
use alloy_zksync::{
    network::unsigned_tx::eip712::PaymasterParams, provider::zksync_provider,
    wallet::ZksyncWallet,
};
use create2_address::create2_address;
use std::{fmt::Debug, str::FromStr};

pub mod create2_address;

#[derive(Debug, Clone)]
pub struct AccountParams {
    pub passkey_expected_origin: String,
}

async fn get_smart_account_bytecode_hash(
    config: &Config,
) -> eyre::Result<FixedBytes<32>> {
    let contracts = config.contracts.clone();
    let provider = {
        let signer =
            PrivateKeySigner::from_str(&config.deploy_wallet.private_key_hex)?;
        let wallet = ZksyncWallet::from(signer);
        let node_url: url::Url = config.clone().node_url;
        let provider = zksync_provider()
            .with_recommended_fillers()
            .wallet(wallet.clone())
            .on_http(node_url.clone());
        let wallet_address = wallet.default_signer().address();
        println!("XDB - Wallet address: {}", wallet_address);
        provider
    };
    let factory = AAFactory::new(contracts.account_factory, &provider);
    let result = factory.beaconProxyBytecodeHash().call().await?;
    println!("XDB get_smart_account_bytecode_hash - result: {:?}", result);
    let hash = result._0;
    println!("XDB get_smart_account_bytecode_hash - hash: {:?}", hash);
    Ok(hash)
}

async fn get_smart_account_proxy_address(
    config: &Config,
) -> eyre::Result<Bytes> {
    let contracts = config.contracts.clone();
    let provider = {
        let signer =
            PrivateKeySigner::from_str(&config.deploy_wallet.private_key_hex)?;
        let wallet = ZksyncWallet::from(signer);
        let node_url: url::Url = config.clone().node_url;
        let provider = zksync_provider()
            .with_recommended_fillers()
            .wallet(wallet.clone())
            .on_http(node_url.clone());
        let wallet_address = wallet.default_signer().address();
        println!("XDB - Wallet address: {}", wallet_address);
        provider
    };
    let factory = AAFactory::new(contracts.account_factory, &provider);
    let result = factory.getEncodedBeacon().call().await?;
    Ok(result._0)
}

async fn is_account_deployed(
    user_id: &str,
    config: &Config,
) -> eyre::Result<bool> {
    let smart_account_address =
        get_smart_account_address_by_user_id(user_id, config).await?;
    let provider = {
        let signer =
            PrivateKeySigner::from_str(&config.deploy_wallet.private_key_hex)?;
        let wallet = ZksyncWallet::from(signer);
        let node_url: url::Url = config.clone().node_url;
        let provider = zksync_provider()
            .with_recommended_fillers()
            .wallet(wallet.clone())
            .on_http(node_url.clone());
        let wallet_address = wallet.default_signer().address();
        println!("XDB - Wallet address: {}", wallet_address);
        provider
    };
    let address_bytecode = provider.get_code_at(smart_account_address).await?;
    Ok(!address_bytecode.is_empty())
}

pub fn get_account_id_by_user_id(
    user_id: &str,
    deploy_wallet_address: &Address,
) -> FixedBytes<32> {
    let combined = {
        let wallet_address_str = deploy_wallet_address.to_string();
        println!(
            "XDB get_account_id_by_user_id - wallet_address_str: {:?}",
            wallet_address_str
        );
        let combined = [user_id, &wallet_address_str].concat();
        println!("XDB get_account_id_by_user_id - combined: {:?}", combined);
        combined
    };
    let hash: FixedBytes<32> = keccak256(combined);
    println!("XDB get_account_id_by_user_id - hash: {:?}", hash);
    hash
}

pub async fn get_smart_account_address_by_user_id(
    user_id: &str,
    config: &Config,
) -> eyre::Result<Address> {
    let contracts = config.contracts.clone();
    check_contract_deployed(
        &config.node_url,
        &Contract {
            address: contracts.account_factory,
            name: "AAFactory".to_string(),
        },
    )
    .await?;
    let account_id =
        get_account_id_by_user_id(user_id, &config.deploy_wallet.address());
    println!(
        "XDB get_smart_account_address_by_user_id - Account ID: {}",
        hex::encode(account_id.as_slice())
    );
    let smart_account_proxy_address =
        get_smart_account_proxy_address(config).await?;
    println!(
        "XDB get_smart_account_address_by_user_id - Smart account proxy address: {}",
        smart_account_proxy_address
    );
    let smart_account_bytecode_hash =
        get_smart_account_bytecode_hash(config).await?;
    println!(
        "XDB get_smart_account_address_by_user_id - Smart account bytecode hash: {}",
        smart_account_bytecode_hash
    );
    let account_address = create2_address(
        contracts.account_factory,
        smart_account_bytecode_hash,
        account_id,
        smart_account_proxy_address,
    );
    println!(
        "XDB get_smart_account_address_by_user_id - Smart account address: {}",
        account_address
    );
    Ok(account_address)
}

async fn deploy_smart_account(
    user_id: &str,
    account_id: &FixedBytes<32>,
    credential: &CredentialDetails,
    account_params: &AccountParams,
    paymaster: Option<PaymasterParams>,
    config: &Config,
) -> eyre::Result<Address> {
    let contracts = config.contracts.clone();

    let args = DeployAccountArgs {
        credential: CredentialDetails {
            id: credential.id.clone(),
            public_key: credential.public_key.clone(),
        },
        // salt: Some(account_id.as_slice().try_into().unwrap()),
        expected_origin: Some(account_params.passkey_expected_origin.clone()),
        unique_account_id: Some(hex::encode(account_id.as_slice())),
        paymaster,
        contracts: contracts.clone(),
        initial_k1_owners: None, // TODO: add initial k1 owners
    };

    let result = deploy_account(args, config).await?;
    let deployed_account_address = result.address;
    println!("Deployed account address: {}", deployed_account_address);

    let derived_address =
        get_smart_account_address_by_user_id(user_id, config).await?;

    if derived_address != deployed_account_address {
        return Err(eyre::eyre!(
            "Deployed address doesn't match derived address"
        ));
    }

    Ok(deployed_account_address)
}

pub async fn create_account(
    user_id: String,
    credential: CredentialDetails,
    account_params: &AccountParams,
    paymaster: Option<PaymasterParams>,
    config: &Config,
) -> eyre::Result<Address> {
    println!("XDB create_account - user_id: {}", user_id);
    println!("XDB create_account - credential: {:?}", credential);
    println!("XDB create_account - account_params: {:?}", account_params);
    println!("XDB create_account - paymaster: {:?}", paymaster);
    println!("XDB create_account - config: {:?}", config);

    let deploy_wallet = config.deploy_wallet.clone();

    let address =
        get_smart_account_address_by_user_id(&user_id, config).await?;
    println!("XDB create_account - address: {}", address);

    let account_id =
        get_account_id_by_user_id(&user_id, &deploy_wallet.address());
    println!("XDB create_account - account_id: {}", account_id);

    let is_already_deployed = is_account_deployed(&user_id, config).await?;
    println!(
        "XDB create_account - is_already_deployed: {}",
        is_already_deployed
    );

    if is_already_deployed {
        println!("XDB create_account - account already deployed");
        return Ok(address);
    }

    _ = deploy_smart_account(
        &user_id,
        &account_id,
        &credential,
        account_params,
        paymaster,
        config,
    )
    .await?;

    println!("XDB create_account - account deployed");

    Ok(address)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::spawn_node_and_deploy_contracts;
    use alloy::primitives::address;

    // Sample COSE key in hex format
    const SAMPLE_COSE_KEY: &[u8] = &[
        0xa5, // map of 5 elements
        0x01, 0x02, // kty: EC2
        0x03, 0x26, // alg: ES256
        0x20, 0x01, // crv: P-256
        0x21, 0x58, 0x20, // x: bytes(32)
        0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
        0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
        0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x22, 0x58,
        0x20, // y: bytes(32)
        0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
        0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
        0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
    ];

    #[tokio::test]
    #[ignore = "This test is currently broken for unknown reasons"]
    async fn test_get_smart_account_bytecode_hash() -> eyre::Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;

        // Act
        let bytecode_hash = get_smart_account_bytecode_hash(&config).await?;

        // Assert
        assert_eq!(bytecode_hash.len(), 32);
        println!("Bytecode hash: 0x{}", hex::encode(bytecode_hash));

        let bytecode_hash2 = get_smart_account_bytecode_hash(&config).await?;
        assert_eq!(bytecode_hash, bytecode_hash2);

        drop(anvil_zksync);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "This test is currently broken due to the account not being deployed to the same address as the expected address"]
    async fn test_create_account() -> eyre::Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;

        let user_id = "unique-base64encoded-string".to_string();
        let credential_public_key = SAMPLE_COSE_KEY.to_vec();
        let account_params = AccountParams {
            passkey_expected_origin: "https://example.com".to_string(),
        };

        let credential_id = user_id.clone();
        let credential = CredentialDetails {
            id: credential_id,
            public_key: credential_public_key.clone(),
        };

        // Act - Create account first time
        let address1 = create_account(
            user_id.clone(),
            credential.clone(),
            &account_params,
            None,
            &config,
        )
        .await?;

        // Assert
        // 1. Address should be valid (20 bytes)
        eyre::ensure!(
            address1.as_slice().len() == 20,
            "Address should be 20 bytes"
        );

        // 2. Account should be deployed
        let is_deployed = is_account_deployed(&user_id, &config).await?;
        eyre::ensure!(is_deployed, "Account should be deployed");

        // 3. Creating account again should return same address (idempotent)
        let address2 = create_account(
            user_id.clone(),
            credential.clone(),
            &account_params,
            None,
            &config,
        )
        .await?;
        eyre::ensure!(
            address1 == address2,
            "Create account should be idempotent"
        );

        // 4. Verify the address matches what we expect from create2
        let expected_address =
            get_smart_account_address_by_user_id(&user_id, &config).await?;
        eyre::ensure!(
            address1 == expected_address,
            "Address should match create2 computation"
        );

        // Cleanup
        drop(anvil_zksync);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "This test is currently broken due address derivation not working"]
    async fn test_get_smart_account_address_by_user_id() -> eyre::Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;
        let contracts = config.contracts.clone();

        let user_id = "test_user_123";
        let deploy_wallet_address =
            address!("1234567890123456789012345678901234567890");

        // Act
        let address1 =
            get_smart_account_address_by_user_id(user_id, &config).await?;

        // Assert
        // 1. Address should be valid (20 bytes)
        assert_eq!(address1.as_slice().len(), 20);

        // 2. Test determinism - same inputs should give same address
        let address2 =
            get_smart_account_address_by_user_id(user_id, &config).await?;
        assert_eq!(
            address1, address2,
            "Address generation should be deterministic"
        );

        // 3. Different user_id should give different address
        let different_address =
            get_smart_account_address_by_user_id("different_user", &config)
                .await?;
        assert_ne!(
            address1, different_address,
            "Different user_id should give different address"
        );

        // 4. Different salt should give different address
        let different_salt_address =
            get_smart_account_address_by_user_id(user_id, &config).await?;
        assert_ne!(
            address1, different_salt_address,
            "Different salt should give different address"
        );

        // 5. Verify components are correctly used
        let account_id =
            get_account_id_by_user_id(user_id, &deploy_wallet_address);
        let bytecode_hash = get_smart_account_bytecode_hash(&config).await?;
        let proxy_address = get_smart_account_proxy_address(&config).await?;

        let expected_address = create2_address(
            contracts.account_factory,
            bytecode_hash,
            account_id,
            proxy_address,
        );
        assert_eq!(
            address1, expected_address,
            "Address should match manual calculation"
        );

        // Cleanup
        drop(anvil_zksync);

        Ok(())
    }
}
