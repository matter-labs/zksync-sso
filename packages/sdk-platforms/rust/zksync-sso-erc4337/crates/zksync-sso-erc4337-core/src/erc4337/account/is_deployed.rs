use alloy::{primitives::Address, providers::Provider};

pub async fn is_smart_account_deployed<P>(
    provider: &P,
    sender_address: Address,
) -> eyre::Result<bool>
where
    P: Provider,
{
    let contract_code = provider.get_code_at(sender_address).await?;

    if contract_code.len() > 2 {
        return Ok(true);
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{node_bindings::Anvil, providers::ProviderBuilder};

    #[tokio::test]
    #[ignore = "implementation not complete"]
    async fn test_is_smart_account_deployed() -> eyre::Result<()> {
        let anvil = Anvil::new().spawn();
        let anvil_url = anvil.endpoint_url();

        let provider = ProviderBuilder::new().connect_http(anvil_url);

        // Test with a known address (anvil's first default account - EOA)
        let eoa_address =
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".parse::<Address>()?;

        // Test that EOA is not deployed as smart contract
        let is_eoa_deployed =
            is_smart_account_deployed(&provider, eoa_address).await?;
        eyre::ensure!(
            !is_eoa_deployed,
            "EOA should not be detected as deployed smart contract"
        );

        // Test with a non-existent address
        let non_existent =
            "0x0000000000000000000000000000000000000123".parse::<Address>()?;
        let is_non_existent_deployed =
            is_smart_account_deployed(&provider, non_existent).await?;
        eyre::ensure!(
            !is_non_existent_deployed,
            "Non-existent address should not be deployed"
        );

        Ok(())
    }
}
