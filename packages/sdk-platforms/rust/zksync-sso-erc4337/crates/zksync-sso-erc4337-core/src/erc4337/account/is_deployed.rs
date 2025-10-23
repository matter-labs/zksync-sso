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
