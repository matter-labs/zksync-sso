use crate::erc4337::entry_point::EntryPoint;
use alloy::{
    contract::{Error, private::Provider},
    primitives::{Address, U256, aliases::U192},
};

pub async fn get_nonce<P>(
    provider: &P,
    address: Address,
    entry_point_address: &Address,
) -> Result<U256, Error>
where
    P: Provider,
{
    get_nonce_with_key(provider, address, entry_point_address, U192::ZERO).await
}

pub async fn get_nonce_with_key<P>(
    provider: &P,
    address: Address,
    entry_point_address: &Address,
    key: U192,
) -> Result<U256, Error>
where
    P: Provider,
{
    let entry_point_instance = EntryPoint::new(*entry_point_address, provider);
    let nonce = entry_point_instance.getNonce(address, key).call().await?;
    Ok(nonce)
}
