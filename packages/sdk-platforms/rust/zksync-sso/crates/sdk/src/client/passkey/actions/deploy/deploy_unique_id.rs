use crate::{
    client::contracts::AAFactory,
    client::passkey::actions::deploy::{
        CredentialDetails, DeployAccountArgs, deploy_account,
    },
    config::{Config, contracts::PasskeyContracts},
    utils::{
        alloy::extensions::ProviderExt,
        contract_deployed::{Contract, check_contract_deployed},
        encoding::paymaster::generate_paymaster_input,
    },
};
use alloy::{
    hex::FromHex,
    network::TransactionBuilder,
    primitives::{Address, Bytes, FixedBytes, U256, address, hex, keccak256},
    providers::Provider,
    signers::local::PrivateKeySigner,
    sol_types::SolEvent,
};
use alloy_zksync::{
    network::{
        receipt_response::ReceiptResponse as ZKReceiptResponse,
        transaction_request::TransactionRequest,
        unsigned_tx::eip712::PaymasterParams,
    },
    provider::zksync_provider,
    wallet::ZksyncWallet,
};
use eyre::{Result, ensure, eyre};
use k256::ecdsa::SigningKey;
use rand::RngCore;
use std::{fmt::Debug, str::FromStr};

const RICH_WALLET_ADDRESS: &str = "f39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
const RICH_WALLET_PRIVATE_KEY_HEX: &str =
    "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

pub async fn get_unique_salt_hex(
    salt: FixedBytes<32>,
    config: &Config,
) -> eyre::Result<String> {
    let (_, deploy_wallet_address) = {
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
        (provider, wallet_address)
    };
    let expected_deploy_wallet_address: Address =
        RICH_WALLET_ADDRESS.to_string().parse()?;
    eyre::ensure!(
        deploy_wallet_address == expected_deploy_wallet_address,
        "XDB - get_unique_salt_hex - Deploy wallet address: {} does not match expected deploy wallet address: {}",
        deploy_wallet_address,
        expected_deploy_wallet_address
    );

    let salt = salt.to_vec();

    let wallet_address_bytes = deploy_wallet_address.0.to_vec();
    println!("XDB wallet_address_bytes: {:?}", wallet_address_bytes);

    let concatenated_bytes = {
        let mut concatenated_bytes = Vec::new();
        concatenated_bytes.extend(salt);
        concatenated_bytes.extend(wallet_address_bytes);
        concatenated_bytes
    };

    println!("XDB concatenated_bytes: {:?}", concatenated_bytes);

    let concatenated_bytes_hex = hex::encode(concatenated_bytes.clone());
    println!("XDB concatenated_bytes_hex: {:?}", concatenated_bytes_hex);

    Ok(concatenated_bytes_hex)
}

pub async fn get_predicted_deployed_account_address(
    salt: FixedBytes<32>,
    config: &Config,
) -> eyre::Result<Address> {
    let (_, deploy_wallet_address) = {
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
        (provider, wallet_address)
    };
    let expected_deploy_wallet_address: Address =
        RICH_WALLET_ADDRESS.to_string().parse()?;
    eyre::ensure!(
        deploy_wallet_address == expected_deploy_wallet_address,
        "XDB - get_predicted_deployed_account_address - Deploy wallet address: {} does not match expected deploy wallet address: {}",
        deploy_wallet_address,
        expected_deploy_wallet_address
    );

    let provider = {
        let node_url: url::Url = config.clone().node_url;
        let provider = zksync_provider()
            .with_recommended_fillers()
            .on_http(node_url.clone());
        provider
    };

    let (aa_factory, aa_factory_address) = {
        let contracts = config.contracts.clone();
        let contract = crate::client::contracts::AAFactory::new(
            contracts.account_factory,
            &provider,
        );
        (contract, contracts.account_factory)
    };

    let bytecode_hash = {
        let result = aa_factory.beaconProxyBytecodeHash().call().await?;
        println!("XDB get_smart_account_bytecode_hash - result: {:?}", result);
        let hash = result._0;
        println!("XDB get_smart_account_bytecode_hash - hash: {:?}", hash);
        hash
    };
    let expected_bytecode_hash = vec![
        1, 0, 0, 251, 169, 193, 69, 186, 126, 6, 245, 59, 209, 96, 113, 241,
        153, 234, 249, 99, 124, 89, 194, 26, 206, 15, 23, 254, 17, 116, 178,
        230,
    ];
    eyre::ensure!(
        bytecode_hash.to_vec() == expected_bytecode_hash,
        "XDB - test_get_predicted_deployed_account_address - Bytecode hash does not match expected bytecode hash"
    );

    let salt = salt.to_vec();

    let wallet_address_bytes = deploy_wallet_address.0.to_vec();
    println!("XDB wallet_address_bytes: {:?}", wallet_address_bytes);

    let concatenated_bytes = {
        let mut concatenated_bytes = Vec::new();
        concatenated_bytes.extend(salt);
        concatenated_bytes.extend(wallet_address_bytes);
        concatenated_bytes
    };
    let expected_concatenated_bytes = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 243, 159, 214, 229, 26, 173, 136, 246, 244,
        206, 106, 184, 130, 114, 121, 207, 255, 185, 34, 102,
    ];
    eyre::ensure!(
        concatenated_bytes == expected_concatenated_bytes,
        "XDB - test_get_predicted_deployed_account_address - Concatenated bytes do not match expected concatenated bytes"
    );

    println!("XDB concatenated_bytes: {:?}", concatenated_bytes);

    let concatenated_bytes_hex = hex::encode(concatenated_bytes.clone());
    println!("XDB concatenated_bytes_hex: {:?}", concatenated_bytes_hex);

    // let expected_concatenated_bytes_hex = "0000000000000000000000000000000000000000000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266";
    // eyre::ensure!(
    //     concatenated_bytes_hex == expected_concatenated_bytes_hex,
    //     "XDB - test_get_predicted_deployed_account_address - Concatenated bytes hex does not match expected concatenated bytes hex"
    // );

    let unique_salt = keccak256(concatenated_bytes.clone());
    println!("XDB unique_salt bytes: {:?}", unique_salt);

    let expected_unique_salt = vec![
        102, 24, 19, 125, 139, 51, 50, 157, 54, 255, 160, 12, 185, 124, 19, 15,
        135, 28, 191, 230, 244, 6, 172, 99, 231, 163, 10, 230, 165, 106, 53,
        15,
    ];
    eyre::ensure!(
        unique_salt.to_vec() == expected_unique_salt,
        "XDB - test_get_predicted_deployed_account_address - Unique salt does not match expected unique salt"
    );

    let unique_salt_hex = hex::encode(unique_salt);
    println!("XDB unique_salt_hex: {:?}", unique_salt_hex);

    let args = {
        let encoded_beacon_return =
            aa_factory.getEncodedBeacon().call().await?;
        println!("XDB encoded_beacon: {:?}", encoded_beacon_return);

        let args = encoded_beacon_return._0;
        println!("XDB args: {:?}", args);

        args
    };
    println!("XDB args: {:?}", args);
    let expected_args = {
        let expected_hex =
            "0000000000000000000000002f446f78886952dbdb873e6088495d42455b51cf"
                .to_string();
        Bytes::from_hex(expected_hex)?
    };
    eyre::ensure!(
        args == expected_args,
        "XDB - test_get_predicted_deployed_account_address - Args {} do not match expected args {}",
        args,
        expected_args
    );

    use crate::client::passkey::account_factory::create2_address::create2_address;
    let standard_create2_address =
        create2_address(aa_factory_address, bytecode_hash, unique_salt, args);

    let expected_standard_create2_address =
        address!("b9055bEeD014D0A91c418De893289FDd4bF50522");
    eyre::ensure!(
        standard_create2_address == expected_standard_create2_address,
        "XDB - test_get_predicted_deployed_account_address - standard_create2_address {} does not match expected {}",
        standard_create2_address,
        expected_standard_create2_address
    );

    println!("XDB standard_create2_address: {}", standard_create2_address);

    Ok(standard_create2_address)
}

// pub async fn get_predicted_deployed_account_address(
//     user_id: String,
//     args: DeployAccountArgs,
//     wallet_address: Address,
//     config: &Config,
// ) -> eyre::Result<Address> {
//     use crate::client::passkey::account_factory::create2_address::create2_address;
//     use crate::client::passkey::account_factory::get_account_id_by_user_id;
//     use crate::client::passkey::account_factory::get_smart_account_bytecode_hash;
//     use crate::client::passkey::account_factory::get_smart_account_proxy_address;

//     let contracts = config.contracts.clone();
//     check_contract_deployed(
//         &config.node_url,
//         &Contract {
//             address: contracts.account_factory,
//             name: "AAFactory".to_string(),
//         },
//     )
//     .await?;
//     let account_id =
//         get_account_id_by_user_id(&user_id, &config.deploy_wallet.address());
//     println!(
//         "XDB get_smart_account_address_by_user_id - Account ID: {}",
//         hex::encode(account_id.as_slice())
//     );
//     let smart_account_proxy_address =
//         get_smart_account_proxy_address(config).await?;
//     println!(
//         "XDB get_smart_account_address_by_user_id - Smart account proxy address: {}",
//         smart_account_proxy_address
//     );
//     let smart_account_bytecode_hash =
//         get_smart_account_bytecode_hash(config).await?;
//     println!(
//         "XDB get_smart_account_address_by_user_id - Smart account bytecode hash: {}",
//         smart_account_bytecode_hash
//     );
//     let account_address = create2_address(
//         contracts.account_factory,
//         smart_account_bytecode_hash,
//         account_id,
//         smart_account_proxy_address,
//     );
//     println!(
//         "XDB get_smart_account_address_by_user_id - Smart account address: {}",
//         account_address
//     );
//     Ok(account_address)
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        client::{
            contracts::AAFactory,
            passkey::actions::deploy::{
                CredentialDetails, DeployAccountArgs, deploy_account,
            },
        },
        config::{Config, contracts::PasskeyContracts},
        utils::{
            alloy::extensions::ProviderExt,
            contract_deployed::{Contract, check_contract_deployed},
            encoding::paymaster::generate_paymaster_input,
            test_utils::{
                spawn_node_and_deploy_contracts,
                zksync_wallet_from_anvil_zksync,
            },
        },
    };

    #[tokio::test]
    async fn test_get_predicted_deployed_account_address() -> Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;

        let (wallet, _, _) = zksync_wallet_from_anvil_zksync(&anvil_zksync)?;

        let wallet_address = wallet.default_signer().address();
        println!("XDB - Wallet address: {}", wallet_address);

        let salt = {
            let mut salt = vec![0; 32];
            salt.fill(0);

            let salt_fixed_bytes = FixedBytes::<32>::from_slice(&salt);

            salt_fixed_bytes
        };

        let predicted_deployed_account_address =
            get_predicted_deployed_account_address(salt, &config).await?;
        println!(
            "XDB - test_get_predicted_deployed_account_address - Predicted deployed account address: {}",
            predicted_deployed_account_address
        );

        drop(anvil_zksync);

        Ok(())
    }

    // #[tokio::test]
    // async fn test_get_predicted_deployed_account_address() -> Result<()> {
    //     use crate::config::deploy_wallet::DeployWallet;

    //     let account_factory =
    //         address!("2BE7068ce5A418a70583A5bCAF5772E26e4F292D");

    //     let passkey = address!("2BE7068ce5A418a70583A5bCAF5772E26e4F292D");
    //     let session = address!("2BE7068ce5A418a70583A5bCAF5772E26e4F292D");
    //     let account_paymaster =
    //         address!("2BE7068ce5A418a70583A5bCAF5772E26e4F292D");
    //     let recovery = address!("2BE7068ce5A418a70583A5bCAF5772E26e4F292D");
    //     let account_proxy =
    //         address!("2BE7068ce5A418a70583A5bCAF5772E26e4F292D");

    //     let passkey_contracts = PasskeyContracts::new(
    //         account_factory,
    //         passkey,
    //         session,
    //         account_paymaster,
    //         recovery,
    //         account_proxy,
    //     );

    //     let deploy_wallet = DeployWallet::new("0x0000000000000000000000000000000000000000000000000000000000000000".to_string())?;

    //     let config = Config::new(
    //         passkey_contracts,
    //         "https://rpc.ankr.com/base-sepolia".parse().unwrap(),
    //         deploy_wallet,
    //     );

    //     let wallet_address = config.deploy_wallet.address();

    //     let user_id = "test-user-id".to_string();

    //     let predicted_deployed_account_address =
    //         get_predicted_deployed_account_address(
    //             user_id.clone(),
    //             DeployAccountArgs::default(),
    //             wallet_address,
    //             &config,
    //         )
    //         .await?;
    //     println!(
    //         "XDB - test_get_predicted_deployed_account_address - Predicted deployed account address: {}",
    //         predicted_deployed_account_address
    //     );
    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_get_predicted_deployed_account_address() -> Result<()> {
    //     use crate::config::deploy_wallet::DeployWallet;

    //     let account_factory =
    //         address!("2BE7068ce5A418a70583A5bCAF5772E26e4F292D");

    //     let passkey = address!("2BE7068ce5A418a70583A5bCAF5772E26e4F292D");
    //     let session = address!("2BE7068ce5A418a70583A5bCAF5772E26e4F292D");
    //     let account_paymaster =
    //         address!("2BE7068ce5A418a70583A5bCAF5772E26e4F292D");
    //     let recovery = address!("2BE7068ce5A418a70583A5bCAF5772E26e4F292D");

    //     let passkey_contracts = PasskeyContracts::new(
    //         account_factory,
    //         passkey,
    //         session,
    //         account_paymaster,
    //         recovery,
    //     );

    //     let deploy_wallet = DeployWallet::new("0x0000000000000000000000000000000000000000000000000000000000000000".to_string())?;

    //     let config = Config::new(
    //         passkey_contracts,
    //         "https://rpc.ankr.com/base-sepolia".parse().unwrap(),
    //         deploy_wallet,
    //     );

    //     let wallet_address = config.deploy_wallet.address();

    //     let user_id = "test-user-id".to_string();

    //     let predicted_deployed_account_address =
    //         get_predicted_deployed_account_address(
    //             user_id.clone(),
    //             DeployAccountArgs::default(),
    //             wallet_address,
    //             &config,
    //         )
    //         .await?;
    //     println!(
    //         "XDB - test_get_predicted_deployed_account_address - Predicted deployed account address: {}",
    //         predicted_deployed_account_address
    //     );
    //     Ok(())
    // }

    #[tokio::test]
    async fn test_deploy_account() -> Result<()> {
        // Arrange
        // let (anvil_zksync, _config, _) =
        //     spawn_node_and_deploy_contracts().await?;

        let node_url: url::Url = "http://localhost:8011".parse()?;

        let contracts = PasskeyContracts {
            account_factory: address!(
                "3081dE6191BAb71C6019fa0Ff4c28a7dC8558481"
            ),
            passkey: address!("a9B93DDcA52eA873eDe192b9b7019de922890E73"),
            session: address!("6b61B6508ceE5e8a58e1DB85217830F9Cefbc3bf"),
            account_paymaster: address!(
                "F2cdeB25478363A8e60B15C7391280D6dD5E630d"
            ),
            recovery: address!("b83b3ac148Fb5d3801E2aDF0A078063C8a06e1Fc"),
        };

        use crate::config::deploy_wallet::DeployWallet;

        let private_key_hex = RICH_WALLET_PRIVATE_KEY_HEX;

        let deploy_wallet = DeployWallet::new(private_key_hex.to_string())?;

        let wallet =
            ZksyncWallet::from(PrivateKeySigner::from_str(&private_key_hex)?);

        let config = Config::new(contracts, node_url.clone(), deploy_wallet);

        // let (wallet, _, _) = zksync_wallet_from_anvil_zksync(&anvil_zksync)?;

        let wallet_address = wallet.default_signer().address();
        println!("XDB - Wallet address: {}", wallet_address);

        let credential_public_key = vec![
            165, 1, 2, 3, 38, 32, 1, 33, 88, 32, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            34, 88, 32, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        ];

        let credential_id = "unique-base64encoded-string".to_string();

        println!(
            "XDB - test_deploy_account_with_initial_k1_owners - Credential ID: {}",
            credential_id
        );

        let deploy_account_credential = CredentialDetails {
            id: credential_id.clone(),
            public_key: credential_public_key,
        };

        let salt = {
            let mut salt = vec![0; 32];
            salt.fill(0);

            let salt_fixed_bytes = FixedBytes::<32>::from_slice(&salt);

            salt_fixed_bytes
        };
        // let unique_account_id = Some(get_unique_salt_hex(salt, &config).await?);
        let unique_account_id = Some(hex::encode(salt.to_vec()));

        let predicted_deployed_account_address =
            get_predicted_deployed_account_address(salt, &config).await?;

        println!(
            "XDB - test_deploy_account - Predicted deployed account address: {}",
            predicted_deployed_account_address
        );

        let contracts = config.clone().contracts;

        let contract_address = contracts.clone().account_factory;
        {
            let factory_contract = Contract {
                address: contract_address,
                name: "MY_AA_FACTORY".to_string(),
            };
            check_contract_deployed(&node_url, &factory_contract).await?;
        };

        let origin: String = "https://example.com".to_string();

        let args = {
            // let paymaster = Some(PaymasterParams {
            //     paymaster: contracts.account_paymaster,
            //     paymaster_input: Bytes::new(),
            // });
            DeployAccountArgs {
                credential: deploy_account_credential,
                expected_origin: Some(origin),
                unique_account_id,
                // paymaster,
                paymaster: None,
                contracts: contracts.clone(),
                initial_k1_owners: Some(vec![wallet_address]),
                ..Default::default()
            }
        };

        let result = deploy_account(args.clone(), &config).await?;

        let deployed_account_address = result.address;

        println!(
            "XDB - test_deploy_account - Deployed account address: {}",
            deployed_account_address
        );

        eyre::ensure!(
            predicted_deployed_account_address == deployed_account_address,
            "XDB - test_deploy_account - Predicted deployed account address: {} does not match deployed account address: {}",
            predicted_deployed_account_address,
            deployed_account_address
        );

        // drop(anvil_zksync);

        Ok(())
    }
}
