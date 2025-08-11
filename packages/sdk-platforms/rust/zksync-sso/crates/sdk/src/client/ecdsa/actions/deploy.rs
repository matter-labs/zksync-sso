use crate::{
    client::modular_account::get_account_created_event,
    config::Config,
    contracts::AAFactory,
    utils::{
        alloy::extensions::ProviderExt,
        contract_deployed::{Contract, check_contract_deployed},
        encoding::{
            ModuleData, encode_module_data,
            paymaster::generate_paymaster_input,
            session::encode_session_key_module_parameters,
        },
        session::session_lib::session_spec::SessionSpec,
    },
};
use alloy::{
    primitives::{Address, Bytes, FixedBytes},
    providers::Provider,
    signers::local::PrivateKeySigner,
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
use eyre::{Result, eyre};
use log::debug;
use rand::RngCore;
use std::{fmt::Debug, str::FromStr};

#[derive(Debug, Clone, Default)]
pub struct Contracts {
    pub account_factory: Address,
    pub session: Address,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeployedAccountDetails {
    pub address: Address,
    pub unique_account_id: FixedBytes<32>,
    pub transaction_receipt: ZKReceiptResponse,
}

#[derive(Debug, Clone, Default)]
pub struct DeployAccountArgs {
    /// Wallet owner
    pub owner: Address,

    /// Unique salt
    pub salt: Option<FixedBytes<32>>,

    /// Vendor prefix
    pub prefix: Option<String>,

    /// Initial session
    pub initial_session: Option<SessionSpec>,

    /// Paymaster used to pay the fees of creating accounts
    pub paymaster: Option<PaymasterParams>,

    /// Contracts
    pub contracts: Contracts,
}

#[allow(dead_code)]
pub async fn deploy_account(
    args: DeployAccountArgs,
    config: &Config,
) -> Result<DeployedAccountDetails> {
    debug!("XDB client::ecdsa::actions::deploy::deploy_account");

    let salt = args.salt.unwrap_or_else(|| {
        let mut random_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut random_bytes);
        random_bytes.into()
    });
    debug!("XDB deploy_account - Using salt: {salt}");

    debug!(
        "XDB deploy_account - paymaster: {:?}",
        args.paymaster
            .as_ref()
            .map(|p| (p.paymaster, alloy::hex::encode(&p.paymaster_input)))
    );
    debug!("XDB deploy_account - contracts: {:?}", args.contracts);

    let owner = args.owner;

    let unique_id: FixedBytes<32> = {
        let prefix = if let Some(prefix) = args.prefix {
            if prefix.len() > 12 {
                return Err(eyre!("prefix must not be longer than 12"));
            } else {
                let prefix_hex = alloy::hex::encode(prefix);

                prefix_hex.as_bytes().to_vec()
            }
        } else {
            // Create a 12 bytes of zeros
            vec![0u8; 12]
        };

        let unique_id: FixedBytes<32> = {
            let prefix_bytes = prefix.as_slice();

            eyre::ensure!(
                prefix_bytes.len() == 12,
                "prefix_bytes.len() must be 12, got {}",
                prefix_bytes.len()
            );

            let owner_bytes = owner.as_slice();

            eyre::ensure!(
                owner_bytes.len() == 20,
                "owner_bytes.len() must be 20, got {}",
                owner_bytes.len()
            );

            let mut bytes_array = [0u8; 32];
            bytes_array[..prefix_bytes.len()].copy_from_slice(prefix_bytes);
            bytes_array[prefix_bytes.len()..].copy_from_slice(owner_bytes);
            bytes_array.into()
        };

        unique_id
    };
    debug!("XDB deploy_account - unique_id: {unique_id}");

    let provider = {
        let node_url: url::Url = config.clone().node_url;

        let wallet = if let Some(deploy_wallet) = config.clone().deploy_wallet {
            ZksyncWallet::from(PrivateKeySigner::from_str(
                &deploy_wallet.private_key_hex,
            )?)
        } else {
            ZksyncWallet::from(PrivateKeySigner::random())
        };

        let provider = zksync_provider()
            .with_recommended_fillers()
            .wallet(wallet.clone())
            .on_http(node_url.clone());
        let wallet_address = wallet.default_signer().address();
        debug!("XDB - Wallet address: {wallet_address}");

        provider
    };

    let account_factory = args.contracts.account_factory;
    debug!("XDB deploy_account - Using factory address: {account_factory}");

    check_contract_deployed(
        &config.node_url.clone(),
        &Contract { address: account_factory, name: "AA_FACTORY".to_string() },
    )
    .await?;

    let encoded_session_key_module_data = args
        .initial_session
        .map(|session| -> Result<Bytes> {
            let encoded_session_parameters =
                encode_session_key_module_parameters(session)?;
            encode_module_data(ModuleData {
                address: args.contracts.session,
                parameters: encoded_session_parameters,
            })
        })
        .transpose()?;

    let mut initial_validators: Vec<Bytes> = Vec::new();
    if let Some(encoded_session_key_module_data) =
        encoded_session_key_module_data
    {
        initial_validators.push(encoded_session_key_module_data);
    }

    let owners = vec![args.owner];

    let factory_instance = AAFactory::new(account_factory, &provider);

    let deploy_call = factory_instance.deployProxySsoAccount(
        unique_id,
        initial_validators.clone(),
        owners.clone(),
    );

    let deploy_tx: TransactionRequest = {
        let mut deploy_tx = deploy_call.into_transaction_request();

        if let Some(mut paymaster) = args.paymaster {
            // If paymaster_input is empty, generate default input
            if paymaster.paymaster_input.is_empty() {
                paymaster.paymaster_input = generate_paymaster_input(None);
            }
            deploy_tx = deploy_tx.with_paymaster_params(paymaster);
        }

        deploy_tx
    };

    debug!("XDB deploy_account - Transaction parameters:");
    debug!("  Unique ID Hash: {unique_id}");
    debug!("  Initial validators: {initial_validators:?}");
    debug!("  Initial owners: {owners:?}");
    debug!("XDB deploy_account - Deploy transaction request: {deploy_tx:?}");

    let tx_hash = provider
        .clone()
        .send_transaction(deploy_tx)
        .await
        .map_err(|e| eyre!("Failed to send transaction: {}", e))?
        .tx_hash()
        .to_owned();

    debug!("XDB deploy_account - Transaction sent with hash: {tx_hash}");

    let transaction_receipt =
        provider.wait_for_transaction_receipt(tx_hash).await?;

    debug!("XDB deploy_account - Transaction receipt: {transaction_receipt:?}");

    let account_created_event =
        get_account_created_event(&transaction_receipt)?;
    let address = account_created_event.accountAddress;
    let unique_account_id = account_created_event.uniqueAccountId;

    debug!("XDB deploy_account - Deployed to address: {address}");

    Ok(DeployedAccountDetails {
        address,
        unique_account_id,
        transaction_receipt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{
        contract_deployed::{Contract, check_contract_deployed},
        test_utils::{
            spawn_node_and_deploy_contracts, zksync_wallet_from_anvil_zksync,
        },
    };
    use alloy::{
        network::TransactionBuilder,
        primitives::{U256, address},
        rpc::types::transaction::TransactionRequest as AlloyTransactionRequest,
        signers::local::LocalSigner,
    };
    use alloy_zksync::{
        network::{transaction_request::TransactionRequest, tx_type::TxType},
        provider::zksync_provider,
    };
    use k256::ecdsa::SigningKey;

    #[tokio::test]
    async fn test_deploy_account() -> Result<()> {
        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;
        let node_url = &config.node_url;

        let (wallet, _, _) = zksync_wallet_from_anvil_zksync(&anvil_zksync)?;

        let wallet_address = wallet.default_signer().address();
        println!("XDB - Wallet address: {wallet_address}");

        let contracts = config.clone().contracts;
        let contract_address = contracts.account_factory;
        {
            let factory_contract = Contract {
                address: contract_address,
                name: "MY_AA_FACTORY".to_string(),
            };
            check_contract_deployed(&node_url.clone(), &factory_contract)
                .await?;
        };

        let args = {
            let salt = None;
            let prefix = None;
            let initial_session = None;
            let paymaster = Some(PaymasterParams {
                paymaster: contracts.account_paymaster,
                paymaster_input: Bytes::new(),
            });

            let contracts = Contracts {
                account_factory: contract_address,
                session: contracts.session,
            };
            DeployAccountArgs {
                owner: wallet_address,
                salt,
                prefix,
                initial_session,
                paymaster,
                contracts,
            }
        };

        let result = deploy_account(args, &config).await?;

        let deployed_account_address = result.address;

        println!(
            "XDB - test_deploy_account - Deployed account address: {deployed_account_address}"
        );

        drop(anvil_zksync);

        Ok(())
    }

    #[tokio::test]
    async fn test_deploy_account_with_initial_k1_owners_and_send_transaction()
    -> Result<()> {
        // Add delay to avoid test run timing out
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

        // Arrange
        let (anvil_zksync, config, _) =
            spawn_node_and_deploy_contracts().await?;
        let node_url = &config.node_url;

        let (mut wallet, _, _) =
            zksync_wallet_from_anvil_zksync(&anvil_zksync)?;

        let wallet_address = wallet.default_signer().address();
        println!("XDB - Wallet address: {wallet_address}");

        let vitalik = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");

        let contracts = config.clone().contracts;
        let contract_address = contracts.account_factory;
        {
            let factory_contract = Contract {
                address: contract_address,
                name: "MY_AA_FACTORY".to_string(),
            };
            check_contract_deployed(&node_url.clone(), &factory_contract)
                .await?;
        };

        let args = {
            let salt = None;
            let prefix = None;
            let initial_session = None;
            let paymaster = Some(PaymasterParams {
                paymaster: contracts.account_paymaster,
                paymaster_input: Bytes::new(),
            });
            let contracts = Contracts {
                account_factory: contract_address,
                session: contracts.session,
            };
            DeployAccountArgs {
                owner: wallet_address,
                salt,
                prefix,
                initial_session,
                paymaster,
                contracts,
            }
        };

        let result = deploy_account(args, &config).await?;

        let deployed_account_address = result.address;

        println!(
            "XDB - test_deploy_account - Deployed account address: {deployed_account_address}"
        );

        {
            // Register the EOA signer for the deployed account
            let credential =
                SigningKey::from(anvil_zksync.keys()[0].to_owned());
            let address = deployed_account_address;
            let local_signer =
                LocalSigner::new_with_credential(credential, address, None);
            wallet.register_signer(local_signer);
            let _ = wallet.signer_by_address(address).unwrap();
        };

        let provider = zksync_provider()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(node_url.clone());

        // Check initial balances
        let vitalik_balance_before = provider.get_balance(vitalik).await?;
        println!(
            "XDB - test_deploy_account_with_initial_k1_owners - Vitalik balance before: {vitalik_balance_before}"
        );
        assert_eq!(vitalik_balance_before, U256::ZERO);

        let account_balance_before =
            provider.get_balance(deployed_account_address).await?;
        println!(
            "XDB - test_deploy_account_with_initial_k1_owners - Account balance before: {account_balance_before}"
        );

        // Fund the account with 0.1 ETH
        let value = U256::from(100000000000000000u64); // 0.1 ETH
        {
            let fund_tx = TransactionRequest::default()
                .with_to(deployed_account_address)
                .with_value(value);
            let fund_tx_hash =
                provider.send_transaction(fund_tx).await?.tx_hash().to_owned();
            let receipt =
                provider.wait_for_transaction_receipt(fund_tx_hash).await?;
            println!(
                "XDB - test_deploy_account_with_initial_k1_owners - Fund receipt: {receipt:?}"
            );
        }
        println!(
            "XDB - test_deploy_account_with_initial_k1_owners - account funded"
        );

        // Verify funding
        let account_balance_after =
            provider.get_balance(deployed_account_address).await?;
        println!("Account balance after funding: {account_balance_after}");
        assert!(account_balance_after == value);

        // Send ETH from smart account to Vitalik
        let send_amount = U256::from(50000000000000000u64); // 0.05 ETH

        // Create the transaction data
        let tx: TransactionRequest = {
            let alloy_tx_request = AlloyTransactionRequest::default()
                .with_from(deployed_account_address)
                .with_to(vitalik)
                .with_value(send_amount);

            let tx_request: TransactionRequest =
                alloy_tx_request.clone().into();
            assert_eq!(tx_request.output_tx_type(), TxType::Eip1559);
            tx_request
        };

        // Send the transaction
        let pending_tx = provider.send_transaction(tx).await?;
        let tx_hash = pending_tx.tx_hash().to_owned();

        // Get receipt
        let receipt = provider.wait_for_transaction_receipt(tx_hash).await?;
        println!("Receipt: {receipt:?}");

        // Verify final balances
        let vitalik_balance_after = provider.get_balance(vitalik).await?;
        println!("Vitalik balance after: {vitalik_balance_after}");
        assert_eq!(vitalik_balance_after, send_amount);

        drop(anvil_zksync);

        Ok(())
    }
}
