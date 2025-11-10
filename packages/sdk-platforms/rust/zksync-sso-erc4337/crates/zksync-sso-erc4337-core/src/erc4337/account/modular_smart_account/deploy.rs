use crate::erc4337::{
    account::modular_smart_account::{
        MSAFactory::{self, deployAccountCall},
        ModularSmartAccount::initializeAccountCall,
        add_passkey::PasskeyPayload,
        session::session_lib::session_spec::SessionSpec,
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
pub struct SessionSigner {
    pub session_spec: SessionSpec,
    pub validator_address: Address,
}

#[derive(Clone)]
pub struct DeployAccountParams<P: Provider + Send + Sync + Clone> {
    pub factory_address: Address,
    pub eoa_signers: Option<EOASigners>,
    pub webauthn_signer: Option<WebAuthNSigner>,
    pub session_signer: Option<SessionSigner>,
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
        session_signer,
        id,
        provider,
    } = params;

    let factory = MSAFactory::new(factory_address, provider.clone());

    let account_id = id.unwrap_or({
        let random_id = {
            use rand::Rng;

            pub fn generate_random_id() -> [u8; 32] {
                let mut random_bytes = [0u8; 32];
                rand::thread_rng().fill(&mut random_bytes);
                random_bytes
            }

            generate_random_id()
        };
        FixedBytes::<32>::from_slice(&random_id)
    });

    let (mut data, mut modules) = if let Some(signers) = eoa_signers {
        let eoa_signer_encoded =
            encode_signers_params(signers.addresses).into();
        let modules = vec![signers.validator_address];
        (vec![eoa_signer_encoded], modules)
    } else {
        (vec![], vec![])
    };

    if let Some(webauthn_signer) = webauthn_signer {
        let webauthn_signer_encoded =
            webauthn_signer.passkey.abi_encode_params().into();
        modules.push(webauthn_signer.validator_address);
        data.push(webauthn_signer_encoded);
    }

    if let Some(session_signer) = session_signer {
        use crate::erc4337::account::modular_smart_account::session::SessionLib::SessionSpec as SessionLibSessionSpec;
        use alloy::sol_types::SolValue;

        // Convert SessionSpec to SessionLib format for encoding
        let session_lib_spec: SessionLibSessionSpec =
            session_signer.session_spec.into();
        let session_encoded: Bytes = session_lib_spec.abi_encode().into();

        modules.push(session_signer.validator_address);
        data.push(session_encoded);
    }

    let init_data: Bytes =
        MSAInitializeAccount::new(modules, data).encode().into();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        erc4337::account::erc7579::module_installed::is_module_installed,
        utils::alloy_utilities::test_utilities::start_anvil_and_deploy_contracts,
    };
    use alloy::primitives::address;

    #[tokio::test]
    async fn test_deploy_account_basic() -> eyre::Result<()> {
        let (_, anvil_instance, provider, contracts, _) =
            start_anvil_and_deploy_contracts().await?;

        let factory_address = contracts.account_factory;

        _ = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: None,
            webauthn_signer: None,
            session_signer: None,
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
            session_signer: None,
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
    async fn test_deploy_account_with_session() -> eyre::Result<()> {
        use crate::erc4337::account::modular_smart_account::session::session_lib::session_spec::{
            SessionSpec, transfer_spec::TransferSpec, usage_limit::UsageLimit, limit_type::LimitType,
        };
        use alloy::primitives::{U256, aliases::U48};

        let (_, anvil_instance, provider, contracts, _) =
            start_anvil_and_deploy_contracts().await?;

        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;
        let session_validator_address = contracts.session_validator;

        // Create EOA signers for deployment authorization
        let signers =
            vec![address!("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")];
        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        // Create a session spec
        let session_signer_address =
            address!("0x70997970C51812dc3A010C7d01b50e0d17dc79C8");
        let transfer_target =
            address!("0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC");

        let session_spec = SessionSpec {
            signer: session_signer_address,
            expires_at: U48::from((1u64 << 48) - 1), // Max U48 value
            fee_limit: UsageLimit {
                limit_type: LimitType::Unlimited,
                limit: U256::ZERO,
                period: U48::ZERO,
            },
            call_policies: vec![],
            transfer_policies: vec![TransferSpec {
                target: transfer_target,
                max_value_per_use: U256::from(1_000_000_000_000_000_000u64), // 1 ETH
                value_limit: UsageLimit {
                    limit_type: LimitType::Lifetime,
                    limit: U256::from(10_000_000_000_000_000_000u64), // 10 ETH lifetime
                    period: U48::ZERO,
                },
            }],
        };

        let session_signer = SessionSigner {
            session_spec,
            validator_address: session_validator_address,
        };

        let address = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: Some(eoa_signers),
            webauthn_signer: None,
            session_signer: Some(session_signer),
            id: None,
            provider: provider.clone(),
        })
        .await?;

        println!("Account deployed with session: {:?}", address);

        // Verify EOA module is installed
        let is_eoa_installed = is_module_installed(
            eoa_validator_address,
            address,
            provider.clone(),
        )
        .await?;
        eyre::ensure!(is_eoa_installed, "EOA module is not installed");

        // Verify session module is installed
        let is_session_installed = is_module_installed(
            session_validator_address,
            address,
            provider.clone(),
        )
        .await?;
        eyre::ensure!(is_session_installed, "Session module is not installed");

        drop(anvil_instance);

        Ok(())
    }
}
