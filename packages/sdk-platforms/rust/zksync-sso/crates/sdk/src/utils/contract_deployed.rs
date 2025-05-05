use crate::config::contracts::PasskeyContracts;
use ::alloy::{primitives::Address, providers::Provider};
use alloy_zksync::provider::zksync_provider;

pub struct Contract {
    pub address: Address,
    pub name: String,
}

pub async fn check_contract_deployed(
    node_url: &url::Url,
    contract: &Contract,
) -> eyre::Result<()> {
    let provider =
        zksync_provider().with_recommended_fillers().on_http(node_url.clone());
    let code = provider.get_code_at(contract.address).await?;
    if code.is_empty() {
        return Err(eyre::eyre!(
            "Contract {} not deployed at address: {}",
            contract.name,
            contract.address
        ));
    }
    println!(
        "Contract {} deployed at address: {}",
        contract.name, contract.address
    );
    Ok(())
}

pub async fn check_contracts_deployed(
    node_url: &url::Url,
    contracts: &PasskeyContracts,
) -> eyre::Result<()> {
    check_contract_deployed(
        node_url,
        &Contract {
            address: contracts.account_factory,
            name: "AAFactory".to_string(),
        },
    )
    .await?;
    check_contract_deployed(
        node_url,
        &Contract { address: contracts.passkey, name: "Passkey".to_string() },
    )
    .await?;
    check_contract_deployed(
        node_url,
        &Contract { address: contracts.session, name: "Session".to_string() },
    )
    .await?;
    Ok(())
}
