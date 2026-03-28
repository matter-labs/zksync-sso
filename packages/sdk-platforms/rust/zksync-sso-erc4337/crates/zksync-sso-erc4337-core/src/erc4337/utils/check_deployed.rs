use crate::config::contracts::Contracts;
use alloy::{primitives::Address, providers::Provider};
use log::debug;

pub struct Contract {
    pub address: Address,
    pub name: String,
}

pub async fn check_contract_deployed<P: Provider + Send + Sync + Clone>(
    contract: &Contract,
    provider: P,
) -> eyre::Result<()> {
    let code = provider.get_code_at(contract.address).await?;
    if code.is_empty() {
        return Err(eyre::eyre!(
            "Contract {} not deployed at address: {}",
            contract.name,
            contract.address
        ));
    }
    debug!(
        "Contract {} deployed at address: {}",
        contract.name, contract.address
    );
    Ok(())
}

pub async fn check_contracts_deployed<P: Provider + Send + Sync + Clone>(
    contracts: &Contracts,
    provider: P,
) -> eyre::Result<()> {
    let contracts = vec![
        Contract {
            address: contracts.entry_point,
            name: "EntryPoint".to_string(),
        },
        Contract {
            address: contracts.eoa_validator,
            name: "EOAKeyValidator".to_string(),
        },
        Contract {
            address: contracts.session_validator,
            name: "SessionKeyValidator".to_string(),
        },
        Contract {
            address: contracts.webauthn_validator,
            name: "WebAuthnValidator".to_string(),
        },
        Contract {
            address: contracts.account_factory,
            name: "MSAFactory".to_string(),
        },
    ];

    for contract in &contracts {
        check_contract_deployed(contract, &provider).await?;
    }

    Ok(())
}
