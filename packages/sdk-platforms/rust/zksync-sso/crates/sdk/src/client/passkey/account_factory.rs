use crate::{
    client::{
        contracts::aa_factory::AAFactory,
        passkey::actions::deploy::{deploy_account, DeployAccountArgs},
    },
    config::Config,
    utils::contract_deployed::{check_contract_deployed, Contract},
};
use alloy::{
    primitives::{keccak256, Address, Bytes, FixedBytes},
    providers::Provider,
};
use alloy_zksync::{
    network::unsigned_tx::eip712::PaymasterParams, provider::zksync_provider,
};
use create2_address::create2_address;

pub mod create2_address;

pub struct AccountParams {
    pub secret_account_salt: String,
    pub passkey_expected_origin: String,
}

async fn get_smart_account_bytecode_hash(
    config: &Config,
) -> eyre::Result<FixedBytes<32>> {
    let contracts = config.contracts.clone();
    let provider = zksync_provider()
        .with_recommended_fillers()
        .on_http(config.node_url.clone());
    let factory = AAFactory::new(contracts.account_factory, &provider);
    let result = factory.beaconProxyBytecodeHash().call().await?;
    Ok(result.beaconProxyBytecodeHash)
}

async fn get_smart_account_proxy_address(
    config: &Config,
) -> eyre::Result<Bytes> {
    let contracts = config.contracts.clone();
    let provider = zksync_provider()
        .with_recommended_fillers()
        .on_http(config.node_url.clone());
    let factory = AAFactory::new(contracts.account_factory, &provider);
    let result = factory.getEncodedBeacon().call().await?;
    Ok(result._0)
}

async fn is_account_deployed(
    user_id: &str,
    secret_account_salt: &str,
    config: &Config,
) -> eyre::Result<bool> {
    let smart_account_address = get_smart_account_address_by_user_id(
        user_id,
        secret_account_salt,
        config,
    )
    .await?;
    let provider = zksync_provider()
        .with_recommended_fillers()
        .on_http(config.node_url.clone());
    let address_bytecode = provider.get_code_at(smart_account_address).await?;
    Ok(!address_bytecode.is_empty())
}

pub fn get_account_id_by_user_id(
    user_id: &str,
    secret_account_salt: &str,
) -> FixedBytes<32> {
    let combined = [user_id, secret_account_salt].concat();
    let hash: FixedBytes<32> = keccak256(combined);
    hash
}

pub async fn get_smart_account_address_by_user_id(
    user_id: &str,
    secret_account_salt: &str,
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
    let account_id = get_account_id_by_user_id(user_id, secret_account_salt);
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
    credential_public_key: &[u8],
    account_params: &AccountParams,
    config: &Config,
) -> eyre::Result<Address> {
    let contracts = config.contracts.clone();

    let paymaster = Some(PaymasterParams {
        paymaster: contracts.account_paymaster,
        paymaster_input: Bytes::new(),
    });

    let args = DeployAccountArgs {
        credential_public_key: credential_public_key.to_vec(),
        salt: Some(account_id.as_slice().try_into().unwrap()),
        expected_origin: Some(account_params.passkey_expected_origin.clone()),
        unique_account_id: Some(account_id.to_string()),
        paymaster,
        contracts: contracts.clone(),
        initial_k1_owners: None, // TODO: add initial k1 owners
    };

    let result = deploy_account(args, config).await?;
    let deployed_account_address = result.address;
    println!("Deployed account address: {}", deployed_account_address);

    let derived_address = get_smart_account_address_by_user_id(
        user_id,
        &account_params.secret_account_salt,
        config,
    )
    .await?;

    if derived_address != deployed_account_address {
        return Err(eyre::eyre!(
            "Deployed address doesn't match derived address"
        ));
    }

    Ok(deployed_account_address)
}

pub async fn create_account(
    user_id: String,
    credential_public_key: Vec<u8>,
    account_params: &AccountParams,
    config: &Config,
) -> eyre::Result<Address> {
    let address = get_smart_account_address_by_user_id(
        &user_id,
        &account_params.secret_account_salt,
        config,
    )
    .await?;

    let account_id = get_account_id_by_user_id(
        &user_id,
        &account_params.secret_account_salt,
    );
    let is_already_deployed = is_account_deployed(
        &user_id,
        &account_params.secret_account_salt,
        config,
    )
    .await?;

    if is_already_deployed {
        return Ok(address);
    }

    _ = deploy_smart_account(
        &user_id,
        &account_id,
        &credential_public_key,
        account_params,
        config,
    )
    .await?;

    Ok(address)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::spawn_node_and_deploy_contracts;

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
    async fn test_create_account() -> eyre::Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;

        let user_id = "test_user_123".to_string();
        let credential_public_key = SAMPLE_COSE_KEY.to_vec();
        let account_params = AccountParams {
            secret_account_salt: "test_salt".to_string(),
            passkey_expected_origin: "https://example.com".to_string(),
        };

        // Act - Create account first time
        let address1 = create_account(
            user_id.clone(),
            credential_public_key.clone(),
            &account_params,
            &config,
        )
        .await?;

        // Assert
        // 1. Address should be valid (20 bytes)
        assert_eq!(address1.as_slice().len(), 20);

        // 2. Account should be deployed
        let is_deployed = is_account_deployed(
            &user_id,
            &account_params.secret_account_salt,
            &config,
        )
        .await?;
        assert!(is_deployed, "Account should be deployed");

        // 3. Creating account again should return same address (idempotent)
        let address2 = create_account(
            user_id.clone(),
            credential_public_key.clone(),
            &account_params,
            &config,
        )
        .await?;
        assert_eq!(address1, address2, "Create account should be idempotent");

        // 4. Verify the address matches what we expect from create2
        let expected_address = get_smart_account_address_by_user_id(
            &user_id,
            &account_params.secret_account_salt,
            &config,
        )
        .await?;
        assert_eq!(
            address1, expected_address,
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
        let contracts = config.contracts.clone();

        let user_id = "test_user_123";
        let secret_salt = "test_salt";

        // Act
        let address1 =
            get_smart_account_address_by_user_id(user_id, secret_salt, &config)
                .await?;

        // Assert
        // 1. Address should be valid (20 bytes)
        assert_eq!(address1.as_slice().len(), 20);

        // 2. Test determinism - same inputs should give same address
        let address2 =
            get_smart_account_address_by_user_id(user_id, secret_salt, &config)
                .await?;
        assert_eq!(
            address1, address2,
            "Address generation should be deterministic"
        );

        // 3. Different user_id should give different address
        let different_address = get_smart_account_address_by_user_id(
            "different_user",
            secret_salt,
            &config,
        )
        .await?;
        assert_ne!(
            address1, different_address,
            "Different user_id should give different address"
        );

        // 4. Different salt should give different address
        let different_salt_address = get_smart_account_address_by_user_id(
            user_id,
            "different_salt",
            &config,
        )
        .await?;
        assert_ne!(
            address1, different_salt_address,
            "Different salt should give different address"
        );

        // 5. Verify components are correctly used
        let account_id = get_account_id_by_user_id(user_id, secret_salt);
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
