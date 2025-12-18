use crate::erc4337::{
    account::{
        erc7579::{
            contract::account::IERC7579Account,
            module::{
                Module,
                installed::{IsModuleInstalledParams, is_module_installed},
            },
        },
        modular_smart_account::send::{SendUserOpParams, send_user_op},
    },
    bundler::pimlico::client::BundlerClient,
    paymaster::params::PaymasterParams,
    signer::Signer,
};
use alloy::{
    primitives::{Address, Bytes},
    providers::Provider,
    sol_types::SolCall,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AddModulePayload {
    pub module: Module,
    pub init_data: Option<Bytes>,
}

impl AddModulePayload {
    pub fn webauthn(address: Address) -> Self {
        Self { module: Module::webauthn_validator(address), init_data: None }
    }

    pub fn session_key(address: Address) -> Self {
        Self { module: Module::session_key_validator(address), init_data: None }
    }

    pub fn guardian(address: Address) -> Self {
        Self { module: Module::guardian_executor(address), init_data: None }
    }
}

#[derive(Clone)]
pub struct AddModuleParams<P: Provider + Send + Sync + Clone> {
    pub account_address: Address,
    pub module: AddModulePayload,
    pub entry_point_address: Address,
    pub paymaster: Option<PaymasterParams>,
    pub provider: P,
    pub bundler_client: BundlerClient,
    pub signer: Signer,
}

pub async fn add_module<P>(params: AddModuleParams<P>) -> eyre::Result<()>
where
    P: Provider + Send + Sync + Clone,
{
    let AddModuleParams {
        account_address,
        module,
        entry_point_address,
        paymaster,
        provider,
        bundler_client,
        signer,
    } = params;

    let payload = module.clone();
    let module = payload.module;
    let module_address = module.address;

    let call_data = add_module_call_data(payload);

    send_user_op(SendUserOpParams {
        account: account_address,
        entry_point: entry_point_address,
        factory_payload: None,
        call_data,
        nonce_key: None,
        paymaster,
        bundler_client,
        provider: provider.clone(),
        signer,
    })
    .await?;

    let is_expected_module_installed =
        is_module_installed(IsModuleInstalledParams {
            module,
            account: account_address,
            provider: provider.clone(),
        })
        .await?;

    eyre::ensure!(
        is_expected_module_installed,
        "{} is not installed",
        module_address
    );

    Ok(())
}

fn add_module_call_data(payload: AddModulePayload) -> Bytes {
    IERC7579Account::installModuleCall {
        moduleTypeId: payload.module.module_type.into(),
        module: payload.module.address,
        initData: payload.init_data.unwrap_or_default(),
    }
    .abi_encode()
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        erc4337::{
            account::modular_smart_account::{
                deploy::{DeployAccountParams, EOASigners, deploy_account},
                test_utilities::fund_account_with_default_amount,
            },
            signer::create_eoa_signer,
        },
        utils::alloy_utilities::test_utilities::{
            config::TestInfraConfig,
            start_node_and_deploy_contracts_and_start_bundler_with_config,
        },
    };
    use alloy::primitives::address;

    #[tokio::test]
    async fn test_add_module() -> eyre::Result<()> {
        let (
            _,
            anvil_instance,
            provider,
            contracts,
            signer_private_key,
            bundler,
            bundler_client,
        ) = {
            let config = TestInfraConfig::rich_wallet_9();
            start_node_and_deploy_contracts_and_start_bundler_with_config(
                &config,
            )
            .await?
        };

        let entry_point_address = contracts.entry_point;

        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;

        let signers =
            vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let account_address = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: Some(eoa_signers),
            webauthn_signer: None,
            session_validator: None,
            id: None,
            provider: provider.clone(),
        })
        .await?;

        println!("Account deployed");

        let is_eoa_module_installed =
            is_module_installed(IsModuleInstalledParams {
                module: Module::validator(eoa_validator_address),
                account: account_address,
                provider: provider.clone(),
            })
            .await?;

        eyre::ensure!(
            is_eoa_module_installed,
            "is_eoa_module_installed is not installed"
        );

        fund_account_with_default_amount(account_address, provider.clone())
            .await?;

        let module_address = contracts.webauthn_validator;

        let signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

        add_module(AddModuleParams {
            account_address,
            module: AddModulePayload::webauthn(module_address),
            entry_point_address,
            paymaster: None,
            provider: provider.clone(),
            bundler_client: bundler_client.clone(),
            signer,
        })
        .await?;

        let is_web_authn_module_installed =
            is_module_installed(IsModuleInstalledParams {
                module: Module::webauthn_validator(module_address),
                account: account_address,
                provider: provider.clone(),
            })
            .await?;

        eyre::ensure!(
            is_web_authn_module_installed,
            "is_web_authn_module is not installed"
        );

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }

    #[tokio::test]
    async fn test_add_guardian_module() -> eyre::Result<()> {
        let (
            _,
            anvil_instance,
            provider,
            contracts,
            signer_private_key,
            bundler,
            bundler_client,
        ) = {
            let config = TestInfraConfig::rich_wallet_9();
            start_node_and_deploy_contracts_and_start_bundler_with_config(
                &config,
            )
            .await?
        };

        let entry_point_address = contracts.entry_point;

        let factory_address = contracts.account_factory;
        let eoa_validator_address = contracts.eoa_validator;

        let signer_address =
            address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720");
        let signers = vec![signer_address];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let account_address = deploy_account(DeployAccountParams {
            factory_address,
            eoa_signers: Some(eoa_signers),
            webauthn_signer: None,
            session_validator: None,
            id: None,
            provider: provider.clone(),
        })
        .await?;

        println!("Account deployed");

        fund_account_with_default_amount(account_address, provider.clone())
            .await?;

        let signer = create_eoa_signer(
            signer_private_key.clone(),
            eoa_validator_address,
        )?;

        let guardian_module = contracts.guardian_executor;

        // Install Guardian module
        add_module(AddModuleParams {
            account_address,
            module: AddModulePayload::guardian(guardian_module),
            entry_point_address,
            paymaster: None,
            provider: provider.clone(),
            bundler_client: bundler_client.clone(),
            signer: signer.clone(),
        })
        .await?;

        drop(anvil_instance);
        drop(bundler);

        Ok(())
    }
}
