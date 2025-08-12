use crate::{
    client::passkey::actions::deploy::{
        CredentialDetails, DeployAccountArgs, deploy_account,
    },
    config::Config,
    contracts::AAFactory,
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
use log::debug;
use std::{fmt::Debug, str::FromStr};

pub mod create2_address;

#[derive(Debug, Clone)]
pub struct AccountParams {
    pub passkey_expected_origin: String,
}

pub(crate) async fn get_smart_account_bytecode_hash(
    config: &Config,
) -> eyre::Result<FixedBytes<32>> {
    let contracts = config.contracts;
    let provider = {
        let node_url: url::Url = config.clone().node_url;

        zksync_provider().with_recommended_fillers().on_http(node_url.clone())
    };
    let factory = AAFactory::new(contracts.account_factory, &provider);
    let result = factory.beaconProxyBytecodeHash().call().await?;
    debug!("XDB get_smart_account_bytecode_hash - result: {result:?}");
    let hash = result._0;
    debug!("XDB get_smart_account_bytecode_hash - hash: {hash:?}");
    Ok(hash)
}

pub(crate) async fn get_smart_account_proxy_address(
    config: &Config,
) -> eyre::Result<Bytes> {
    let contracts = config.contracts;
    let provider = {
        let node_url: url::Url = config.clone().node_url;

        zksync_provider().with_recommended_fillers().on_http(node_url.clone())
    };
    let factory = AAFactory::new(contracts.account_factory, &provider);
    let result = factory.getEncodedBeacon().call().await?;
    Ok(result._0)
}

async fn is_account_deployed(
    user_id: String,
    config: &Config,
) -> eyre::Result<bool> {
    let smart_account_address =
        get_smart_account_address_by_user_id(user_id, config).await?;
    let provider = {
        let node_url: url::Url = config.clone().node_url;

        zksync_provider().with_recommended_fillers().on_http(node_url.clone())
    };
    let address_bytecode = provider.get_code_at(smart_account_address).await?;
    Ok(!address_bytecode.is_empty())
}

pub fn get_account_id_by_user_id(user_id: String) -> FixedBytes<32> {
    debug!("XDB get_account_id_by_user_id - user_id: {user_id}");

    let salt = alloy::hex::encode(user_id);
    debug!("XDB salt: {salt:?}");

    let salt_hash = keccak256(salt);
    debug!("XDB salt_hash: {salt_hash:?}");

    salt_hash
}

pub async fn get_smart_account_address_by_user_id(
    user_id: String,
    config: &Config,
) -> eyre::Result<Address> {
    let contracts = config.contracts;
    check_contract_deployed(
        &config.node_url,
        &Contract {
            address: contracts.account_factory,
            name: "AAFactory".to_string(),
        },
    )
    .await?;
    let unique_id = get_account_id_by_user_id(user_id.clone());
    debug!("XDB get_smart_account_address_by_user_id - unique_id: {unique_id}");

    let smart_account_proxy_address =
        get_smart_account_proxy_address(config).await?;
    debug!(
        "XDB get_smart_account_address_by_user_id - Smart account proxy address: {smart_account_proxy_address}"
    );

    let smart_account_bytecode_hash =
        get_smart_account_bytecode_hash(config).await?;
    debug!(
        "XDB get_smart_account_address_by_user_id - Smart account bytecode hash: {smart_account_bytecode_hash}"
    );

    let account_id_hash = unique_id;

    let deploy_wallet_address = {
        let wallet = if let Some(deploy_wallet) = config.clone().deploy_wallet {
            ZksyncWallet::from(PrivateKeySigner::from_str(
                &deploy_wallet.private_key_hex,
            )?)
        } else {
            ZksyncWallet::from(PrivateKeySigner::random())
        };

        wallet.default_signer().address()
    };

    let wallet_address_bytes = deploy_wallet_address.0.to_vec();
    debug!(
        "XDB get_predicted_deployed_account_address - wallet_address_bytes: {wallet_address_bytes:?}"
    );

    let concatenated_bytes = {
        let mut concatenated_bytes = Vec::new();
        concatenated_bytes.extend(account_id_hash.to_vec());
        concatenated_bytes.extend(wallet_address_bytes);
        concatenated_bytes
    };
    debug!(
        "XDB get_predicted_deployed_account_address - concatenated_bytes: {concatenated_bytes:?}"
    );

    let concatenated_bytes_hex = alloy::hex::encode(concatenated_bytes.clone());
    debug!(
        "XDB get_predicted_deployed_account_address - concatenated_bytes_hex: {concatenated_bytes_hex:?}"
    );

    let unique_salt = keccak256(concatenated_bytes.clone());
    debug!(
        "XDB get_predicted_deployed_account_address - unique_salt: {unique_salt:?}"
    );

    let unique_salt_hex = alloy::hex::encode(unique_salt);
    debug!(
        "XDB get_predicted_deployed_account_address - unique_salt_hex: {unique_salt_hex:?}"
    );

    let address = create2_address(
        contracts.account_factory,
        smart_account_bytecode_hash,
        unique_salt,
        smart_account_proxy_address,
    );
    debug!(
        "XDB get_smart_account_address_by_user_id - Smart account address: {address}"
    );

    Ok(address)
}

async fn deploy_smart_account(
    user_id: String,
    credential: &CredentialDetails,
    account_params: &AccountParams,
    paymaster: Option<PaymasterParams>,
    config: &Config,
) -> eyre::Result<Address> {
    let contracts = config.contracts;

    let args = DeployAccountArgs {
        credential: CredentialDetails {
            id: credential.id.clone(),
            public_key: credential.public_key.clone(),
        },
        expected_origin: Some(account_params.passkey_expected_origin.clone()),
        unique_account_id: Some(user_id.clone()),
        paymaster,
        contracts,
        initial_k1_owners: None,
        initial_session: None,
    };

    let result = deploy_account(args, config).await?;
    let deployed_account_address = result.address;
    debug!("Deployed account address: {deployed_account_address}");

    let derived_address =
        get_smart_account_address_by_user_id(user_id.clone(), config).await?;

    if derived_address != deployed_account_address {
        return Err(eyre::eyre!(
            "Deployed address {} doesn't match derived address: {}",
            deployed_account_address,
            derived_address
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
    debug!("XDB create_account - user_id: {user_id}");
    debug!("XDB create_account - credential: {credential:?}");
    debug!("XDB create_account - account_params: {account_params:?}");
    debug!("XDB create_account - paymaster: {paymaster:?}");
    debug!("XDB create_account - config: {config:?}");

    let address =
        get_smart_account_address_by_user_id(user_id.clone(), config).await?;
    debug!("XDB create_account - address: {address}");

    let is_already_deployed =
        is_account_deployed(user_id.clone(), config).await?;
    debug!("XDB create_account - is_already_deployed: {is_already_deployed}");

    if is_already_deployed {
        debug!("XDB create_account - account already deployed");
        return Ok(address);
    }

    _ = deploy_smart_account(
        user_id.clone(),
        &credential,
        account_params,
        paymaster,
        config,
    )
    .await?;

    debug!("XDB create_account - account deployed");

    Ok(address)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::{
        passkey::get_mock_credential_details, spawn_node_and_deploy_contracts,
    };
    use alloy::signers::local::PrivateKeySigner;

    #[tokio::test]
    async fn test_get_smart_account_bytecode_hash() -> eyre::Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;

        // Act
        let bytecode_hash = get_smart_account_bytecode_hash(&config).await?;

        // Assert
        assert_eq!(bytecode_hash.len(), 32);
        println!("Bytecode hash: 0x{}", alloy::hex::encode(bytecode_hash));

        let bytecode_hash2 = get_smart_account_bytecode_hash(&config).await?;
        assert_eq!(bytecode_hash, bytecode_hash2);

        drop(anvil_zksync);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_account() -> eyre::Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;

        let user_id = "unique-id".to_string();

        let contracts = config.clone().contracts;

        let origin: String = "https://example.com".to_string();

        let account_params = AccountParams { passkey_expected_origin: origin };

        let credential = get_mock_credential_details();

        let paymaster = Some(PaymasterParams {
            paymaster: contracts.account_paymaster,
            paymaster_input: Bytes::new(),
        });

        // Act - Create account first time
        let address1 = create_account(
            user_id.clone(),
            credential.clone(),
            &account_params,
            paymaster,
            &config,
        )
        .await?;

        // Assert
        // 1. Account should be deployed
        let is_deployed = is_account_deployed(user_id.clone(), &config).await?;
        eyre::ensure!(is_deployed, "Account should be deployed");

        // 2. Creating account again should return same address (idempotent)
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
            get_smart_account_address_by_user_id(user_id.clone(), &config)
                .await?;
        eyre::ensure!(
            address1 == expected_address,
            "Address should match create2 computation"
        );

        // Cleanup
        drop(anvil_zksync);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_smart_account_address_by_user_id() -> eyre::Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;

        let wallet_address = {
            let wallet = ZksyncWallet::from(PrivateKeySigner::from_str(
                &config.clone().deploy_wallet.unwrap().private_key_hex,
            )?);
            wallet.default_signer().address()
        };
        println!("XDB - Wallet address: {wallet_address}");

        let account_factory_address = config.contracts.account_factory;
        println!("XDB - Account factory address: {account_factory_address}");

        // Act
        let user_id = "unique-id".to_string();
        let address1 =
            get_smart_account_address_by_user_id(user_id.clone(), &config)
                .await?;

        // Assert
        // 1. Address should be valid (20 bytes)
        eyre::ensure!(
            address1.as_slice().len() == 20,
            "Address should be 20 bytes"
        );

        // 2. Test determinism - same inputs should give same address
        let address2 =
            get_smart_account_address_by_user_id(user_id.clone(), &config)
                .await?;
        eyre::ensure!(
            address1 == address2,
            "Address generation should be deterministic"
        );

        // 3. Different user_id should give different address
        let different_address = get_smart_account_address_by_user_id(
            "different_user".to_string(),
            &config,
        )
        .await?;
        eyre::ensure!(
            address1 != different_address,
            "Different user_id should give different address"
        );

        // 4. Verify components are correctly used

        let account_id_hash = get_account_id_by_user_id(user_id.clone());
        let deploy_wallet_address = {
            let signer = PrivateKeySigner::from_str(
                &config.clone().deploy_wallet.unwrap().private_key_hex,
            )?;
            let wallet = ZksyncWallet::from(signer);

            wallet.default_signer().address()
        };
        let wallet_address_bytes = deploy_wallet_address.0.to_vec();
        let concatenated_bytes = {
            let mut concatenated_bytes = Vec::new();
            concatenated_bytes.extend(account_id_hash.to_vec());
            concatenated_bytes.extend(wallet_address_bytes);
            concatenated_bytes
        };
        let unique_salt = keccak256(concatenated_bytes.clone());
        let bytecode_hash = get_smart_account_bytecode_hash(&config).await?;
        let proxy_address = get_smart_account_proxy_address(&config).await?;
        let expected_address = create2_address(
            account_factory_address,
            bytecode_hash,
            unique_salt,
            proxy_address,
        );
        eyre::ensure!(
            address1 == expected_address,
            "Address should match manual calculation"
        );

        // Cleanup
        drop(anvil_zksync);

        Ok(())
    }
}
