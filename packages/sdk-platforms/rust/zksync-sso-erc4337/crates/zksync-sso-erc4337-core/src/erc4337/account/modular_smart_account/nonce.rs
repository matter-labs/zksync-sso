use crate::erc4337::entry_point::EntryPoint;
use alloy::{
    primitives::{Address, U256, Uint},
    providers::Provider,
};

pub async fn get_nonce<P: Provider + Send + Sync + Clone>(
    entry_point: Address,
    account_address: Address,
    nonce_key: Uint<192, 3>,
    provider: P,
) -> eyre::Result<U256> {
    let entry_point = EntryPoint::new(entry_point, provider.clone());
    let nonce = entry_point.getNonce(account_address, nonce_key).call().await?;
    Ok(nonce)
}
