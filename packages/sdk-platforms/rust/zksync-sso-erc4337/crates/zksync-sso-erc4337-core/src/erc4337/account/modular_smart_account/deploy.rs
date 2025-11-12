pub mod user_op;

use crate::erc4337::{
    account::modular_smart_account::{
        MSAFactory::{self, deployAccountCall},
        ModularSmartAccount::initializeAccountCall,
        add_passkey::PasskeyPayload,
    },
    utils::check_deployed::{Contract, check_contract_deployed},
};
use alloy::{
    primitives::{Address, Bytes, FixedBytes},
    providers::Provider,
    rpc::types::TransactionReceipt,
    sol,
    sol_types::{SolCall, SolEvent, SolValue},
};
use rand::Rng;

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
        Self(deployAccountCall { salt: account_id, initData: init_data })
    }

    pub fn encode(&self) -> Vec<u8> {
        deployAccountCall::abi_encode(&self.0)
    }
}

fn encode_signers_params(signers: Vec<Address>) -> Vec<u8> {
    SignersParams { signers: signers.to_vec() }.abi_encode_params()
}

#[derive(Clone)]
pub struct EOASigners {
    pub addresses: Vec<Address>,
    pub validator_address: Address,
}

#[derive(Clone)]
pub struct WebAuthNSigner {
    pub passkey: PasskeyPayload,
    pub validator_address: Address,
}

#[derive(Clone)]
pub struct SessionValidatorConfig {
    pub validator_address: Address,
    // Optional: If you want to install initial sessions during deployment,
    // add initial_sessions: Vec<SessionSpec> here later
}

#[derive(Clone)]
pub struct DeployAccountParams<P: Provider + Send + Sync + Clone> {
    pub factory_address: Address,
    pub eoa_signers: Option<EOASigners>,
    pub webauthn_signer: Option<WebAuthNSigner>,
    pub session_validator: Option<SessionValidatorConfig>,
    pub id: Option<FixedBytes<32>>,
    pub provider: P,
}

pub async fn deploy_account<P>(
    params: DeployAccountParams<P>,
) -> eyre::Result<Address>
where
    P: Provider + Send + Sync + Clone,
{
    let DeployAccountParams {
        factory_address,
        eoa_signers,
        webauthn_signer,
        session_validator,
        id,
        provider,
    } = params;

    let account_id = id.unwrap_or(generate_random_account_id());

    let init_data =
        create_init_data(eoa_signers, webauthn_signer, session_validator);

    let factory = MSAFactory::new(factory_address, provider.clone());
    let deploy_account = factory.deployAccount(account_id, init_data);

    let receipt = deploy_account.send().await?.get_receipt().await?;

    let address = get_account_created_address(&receipt)?;

    check_contract_deployed(
        &Contract { address, name: "ModularSmartAccount".to_string() },
        provider,
    )
    .await?;

    Ok(address)
}

fn get_account_created_address(
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

fn create_init_data(
    eoa_signers: Option<EOASigners>,
    webauthn_signer: Option<WebAuthNSigner>,
    session_validator: Option<SessionValidatorConfig>,
) -> Bytes {
    let (data, modules) =
        modules_from_signers(eoa_signers, webauthn_signer, session_validator);
    MSAInitializeAccount::new(modules, data).encode().into()
}

fn modules_from_signers(
    eoa_signers: Option<EOASigners>,
    webauthn_signer: Option<WebAuthNSigner>,
    session_validator: Option<SessionValidatorConfig>,
) -> (Vec<Bytes>, Vec<Address>) {
    let (mut data, mut modules) = modules_from_eoa_signers(eoa_signers);

    if let Some(webauthn_signer) = webauthn_signer {
        let webauthn_signer_encoded =
            webauthn_signer.passkey.abi_encode_params().into();
        modules.push(webauthn_signer.validator_address);
        data.push(webauthn_signer_encoded);
    }

    // Add session validator if provided
    if let Some(session_config) = session_validator {
        // Session validator module doesn't require initialization data
        // (sessions are created separately after deployment)
        modules.push(session_config.validator_address);
        data.push(Bytes::new()); // Empty initialization data
    }

    (data, modules)
}

fn modules_from_eoa_signers(
    eoa_signers: Option<EOASigners>,
) -> (Vec<Bytes>, Vec<Address>) {
    let Some(signers) = eoa_signers else {
        return (vec![], vec![]);
    };

    let eoa_signer_encoded = encode_signers_params(signers.addresses).into();

    let modules = vec![signers.validator_address];

    (vec![eoa_signer_encoded], modules)
}

fn generate_random_account_id() -> FixedBytes<32> {
    let random_id = generate_random_id();
    FixedBytes::<32>::from_slice(&random_id)
}

fn generate_random_id() -> [u8; 32] {
    let mut random_bytes = [0u8; 32];
    rand::thread_rng().fill(&mut random_bytes);
    random_bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        erc4337::account::erc7579::module_installed::is_module_installed,
        utils::alloy_utilities::test_utilities::start_anvil_and_deploy_contracts,
    };
    use alloy::primitives::{U256, Uint, address};

    #[tokio::test]
    async fn test_deploy_account_basic() -> eyre::Result<()> {
        let (_, anvil_instance, provider, contracts, _) =
            start_anvil_and_deploy_contracts().await?;

        let factory_address = contracts.account_factory;

        _ = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: None,
            webauthn_signer: None,
            session_validator: None,
            id: None,
            provider: provider.clone(),
        })
        .await?;

        drop(anvil_instance);

        Ok(())
    }

    #[tokio::test]
    async fn test_deploy_account_with_eoa_signer() -> eyre::Result<()> {
        let (_, anvil_instance, provider, contracts, _) =
            start_anvil_and_deploy_contracts().await?;

        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;

        let signers =
            vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let address = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: Some(eoa_signers),
            webauthn_signer: None,
            session_validator: None,
            id: None,
            provider: provider.clone(),
        })
        .await?;

        println!("Account deployed");

        let is_module_installed = is_module_installed(
            eoa_validator_address,
            address,
            provider.clone(),
        )
        .await?;
        eyre::ensure!(is_module_installed, "Module is not installed");

        drop(anvil_instance);

        Ok(())
    }

    #[tokio::test]
    async fn test_deploy_account_with_session_validator() -> eyre::Result<()> {
        let (_, anvil_instance, provider, contracts, _) =
            start_anvil_and_deploy_contracts().await?;

        let factory_address = contracts.account_factory;
        let session_validator_address = contracts.session_validator;

        let session_config = SessionValidatorConfig {
            validator_address: session_validator_address,
        };

        let address = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: None,
            webauthn_signer: None,
            session_validator: Some(session_config),
            id: None,
            provider: provider.clone(),
        })
        .await?;

        println!("Account deployed with session validator");

        let is_module_installed = is_module_installed(
            session_validator_address,
            address,
            provider.clone(),
        )
        .await?;
        eyre::ensure!(
            is_module_installed,
            "Session validator module is not installed"
        );

        drop(anvil_instance);

        Ok(())
    }

    #[tokio::test]
    async fn test_deploy_account_with_eoa_and_session() -> eyre::Result<()> {
        let (_, anvil_instance, provider, contracts, _) =
            start_anvil_and_deploy_contracts().await?;

        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;
        let session_validator_address = contracts.session_validator;

        let signers =
            vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let session_config = SessionValidatorConfig {
            validator_address: session_validator_address,
        };

        let address = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: Some(eoa_signers),
            webauthn_signer: None,
            session_validator: Some(session_config),
            id: None,
            provider: provider.clone(),
        })
        .await?;

        println!("Account deployed with EOA and session validators");

        // Verify EOA validator is installed
        let is_eoa_installed = is_module_installed(
            eoa_validator_address,
            address,
            provider.clone(),
        )
        .await?;
        eyre::ensure!(
            is_eoa_installed,
            "EOA validator module is not installed"
        );

        // Verify session validator is installed
        let is_session_installed = is_module_installed(
            session_validator_address,
            address,
            provider.clone(),
        )
        .await?;
        eyre::ensure!(
            is_session_installed,
            "Session validator module is not installed"
        );

        drop(anvil_instance);

        Ok(())
    }

    /// Test comprehensive session flow: deploy with session validator, create session, and transact
    #[tokio::test]
    async fn test_deploy_with_session_and_transact() -> eyre::Result<()> {
        use crate::erc4337::{
            account::{
                erc7579::{Execution, calls::encode_calls},
                modular_smart_account::{
                    send::{SendParams, send_transaction},
                    session::{
                        create::create_session,
                        send::keyed_nonce,
                        session_lib::session_spec::{
                            SessionSpec, limit_type::LimitType,
                            transfer_spec::TransferSpec,
                            usage_limit::UsageLimit,
                        },
                        signature::session_signature,
                    },
                    test_utilities::fund_account_with_default_amount,
                },
            },
            signer::{Signer, create_eoa_signer},
        };
        use crate::utils::alloy_utilities::test_utilities::{
            TestInfraConfig,
            start_anvil_and_deploy_contracts_and_start_bundler_with_config,
        };
        use alloy::signers::local::PrivateKeySigner;
        use std::{future::Future, pin::Pin, str::FromStr, sync::Arc};

        // Start test infrastructure
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

        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;
        let session_validator_address = contracts.session_validator;
        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        // Generate session key
        let session_key_hex = "0xb1da23908ba44fb1c6147ac1b32a1dbc6e7704ba94ec495e588d1e3cdc7ca6f9";
        let session_signer_address =
            PrivateKeySigner::from_str(session_key_hex)?.address();
        println!("Session signer address: {}", session_signer_address);

        // Deploy account WITH session validator pre-installed
        let signers =
            vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];
        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };
        let session_config = SessionValidatorConfig {
            validator_address: session_validator_address,
        };

        let address = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: Some(eoa_signers),
            webauthn_signer: None,
            session_validator: Some(session_config),
            id: None,
            provider: provider.clone(),
        })
        .await?;

        println!("✓ Account deployed with session validator pre-installed");

        // Verify both modules are installed
        let is_eoa_installed = is_module_installed(
            eoa_validator_address,
            address,
            provider.clone(),
        )
        .await?;
        eyre::ensure!(is_eoa_installed, "EOA validator not installed");

        let is_session_installed = is_module_installed(
            session_validator_address,
            address,
            provider.clone(),
        )
        .await?;
        eyre::ensure!(is_session_installed, "Session validator not installed");

        println!("✓ Both EOA and Session validators verified as installed");

        // Fund the account
        fund_account_with_default_amount(address, provider.clone()).await?;
        println!("✓ Account funded");

        // Create a session using EOA signer
        let eoa_signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

        let expires_at = Uint::from(2088558400u64); // Year 2036
        let target = address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");

        let session_spec = SessionSpec {
            signer: session_signer_address,
            expires_at,
            call_policies: vec![],
            fee_limit: UsageLimit {
                limit_type: LimitType::Lifetime,
                limit: U256::from(1_000_000_000_000_000_000u64), // 1 ETH
                period: Uint::from(0),
            },
            transfer_policies: vec![TransferSpec {
                max_value_per_use: U256::from(1_000_000_000_000_000u64), // 0.001 ETH
                target,
                value_limit: UsageLimit {
                    limit_type: LimitType::Unlimited,
                    limit: U256::from(0),
                    period: Uint::from(0),
                },
            }],
        };

        create_session(
            address,
            session_spec.clone(),
            entry_point_address,
            session_validator_address,
            bundler_client.clone(),
            provider.clone(),
            eoa_signer,
        )
        .await?;

        println!("✓ Session created successfully");

        // Send transaction using the session key
        let call = {
            let value = U256::from(1_000_000_000_000_000u64); // 0.001 ETH
            let data = Bytes::default();
            Execution { target, value, data }
        };

        let calls = vec![call];
        let calldata = encode_calls(calls).into();

        // Create session signer
        let session_key_hex_owned = session_key_hex.to_string();
        let session_spec_arc = Arc::new(session_spec.clone());

        let session_signer = {
            let stub_sig = session_signature(
                &session_key_hex_owned,
                session_validator_address,
                &session_spec,
                Default::default(),
            )?;

            let session_key_hex_arc = Arc::new(session_key_hex_owned.clone());
            let session_spec_arc_inner = Arc::clone(&session_spec_arc);

            let signature_provider = Arc::new(
                move |hash: FixedBytes<32>| -> Pin<
                    Box<dyn Future<Output = eyre::Result<Bytes>> + Send>,
                > {
                    let session_key_hex = Arc::clone(&session_key_hex_arc);
                    let session_spec = Arc::clone(&session_spec_arc_inner);
                    Box::pin(async move {
                        session_signature(
                            session_key_hex.as_str(),
                            session_validator_address,
                            &session_spec,
                            hash,
                        )
                    })
                        as Pin<
                            Box<
                                dyn Future<Output = eyre::Result<Bytes>> + Send,
                            >,
                        >
                },
            );

            Signer { provider: signature_provider, stub_signature: stub_sig }
        };

        let keyed_nonce = keyed_nonce(session_signer_address);

        send_transaction(SendParams {
            account: address,
            entry_point: entry_point_address,
            factory_payload: None,
            call_data: calldata,
            nonce_key: Some(keyed_nonce),
            paymaster: None,
            bundler_client,
            provider,
            signer: session_signer,
        })
        .await?;

        println!("✓ Transaction sent successfully using session key");
        println!("\n✅ Complete session flow validated:");
        println!("   1. Deploy account with session validator");
        println!("   2. Create session with EOA signer");
        println!("   3. Send transaction using session key");

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }
}
