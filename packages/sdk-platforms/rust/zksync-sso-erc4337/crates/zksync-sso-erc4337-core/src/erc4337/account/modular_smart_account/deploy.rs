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

// For encoding module data compatible with deployProxySsoAccount
sol! {
    struct ModuleData {
        address module_address;
        bytes parameters;
    }
}

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

pub struct EOASigners {
    pub addresses: Vec<Address>,
    pub validator_address: Address,
}

pub struct WebauthNSigner {
    pub passkey: PasskeyPayload,
    pub validator_address: Address,
}

pub async fn deploy_account<P: Provider + Send + Sync + Clone>(
    factory_address: Address,
    eoa_signers: Option<EOASigners>,
    webauthn_signer: Option<WebauthNSigner>,
    provider: P,
) -> eyre::Result<Address> {
    let factory = MSAFactory::new(factory_address, provider.clone());

    let random_id = {
        use rand::Rng;

        pub fn generate_random_id() -> [u8; 32] {
            let mut random_bytes = [0u8; 32];
            rand::thread_rng().fill(&mut random_bytes);
            random_bytes
        }

        generate_random_id()
    };

    let account_id = FixedBytes::<32>::from_slice(&random_id);

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

// ===== ENCODING FUNCTIONS FOR TYPESCRIPT SDK =====

/// Encode complete calldata for deployProxySsoAccount
/// This is the single function that should be used from TypeScript
///
/// # Parameters
/// * `account_id` - Unique account ID (bytes32)
/// * `passkey` - Optional passkey payload
/// * `passkey_validator` - Address of passkey validator (required if passkey is provided)
/// * `session_validator` - Address of session validator
/// * `session_data` - Optional session initialization data (already encoded)
/// * `recovery_validator` - Address of recovery validator
/// * `oidc_recovery_validator` - Address of OIDC recovery validator
///
/// # Returns
/// Complete calldata for calling deployProxySsoAccount on the factory
pub fn encode_deploy_account_call_data(
    account_id: FixedBytes<32>,
    passkey: Option<PasskeyPayload>,
    passkey_validator: Option<Address>,
    session_validator: Address,
    session_data: Option<Bytes>,
    recovery_validator: Address,
    oidc_recovery_validator: Address,
) -> Bytes {
    let mut modules: Vec<Address> = Vec::new();
    let mut data: Vec<Bytes> = Vec::new();

    // Add passkey validator if provided
    if let Some(passkey_payload) = passkey {
        let passkey_params = passkey_payload.abi_encode_params();
        modules.push(passkey_validator.expect("passkey_validator required when passkey is provided"));
        data.push(passkey_params.into());
    }

    // Add session validator
    let session_params = session_data.unwrap_or_else(|| Bytes::new());
    modules.push(session_validator);
    data.push(session_params);

    // Add guardian recovery validator
    modules.push(recovery_validator);
    data.push(Bytes::new());

    // Add OIDC recovery validator
    modules.push(oidc_recovery_validator);
    data.push(Bytes::new());

    // Encode the complete call to deployAccount
    // deployAccount(bytes32 accountId, bytes initData)
    // The initData contains the MSAInitializeAccount call
    let init_data: Bytes = MSAInitializeAccount::new(modules, data).encode().into();

    MSAFactory::deployAccountCall {
        accountId: account_id,
        initData: init_data,
    }
    .abi_encode()
    .into()
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

        _ = deploy_account(factory_address, None, None, provider.clone())
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

        let address = deploy_account(
            factory_address,
            Some(eoa_signers),
            None,
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

        drop(anvil_instance);

        Ok(())
    }
}
