use crate::erc4337::entry_point::contract::EntryPoint;
use alloy::{
    contract::{Error, private::Provider},
    primitives::{Address, Bytes, U256, aliases::U192},
    sol_types::SolCall,
};

pub async fn get_nonce<P>(
    sender: Address,
    entry_point: Address,
    provider: P,
) -> Result<U256, Error>
where
    P: Provider + Send + Sync + Clone,
{
    get_nonce_with_key(GetNonceWithKeyParams {
        sender,
        entry_point,
        key: U192::ZERO,
        provider,
    })
    .await
}

#[derive(Clone)]
pub struct GetNonceWithKeyParams<P: Provider + Send + Sync + Clone> {
    pub sender: Address,
    pub entry_point: Address,
    pub key: U192,
    pub provider: P,
}

pub async fn get_nonce_with_key<P>(
    params: GetNonceWithKeyParams<P>,
) -> Result<U256, Error>
where
    P: Provider + Send + Sync + Clone,
{
    let GetNonceWithKeyParams { sender, entry_point, key, provider } = params;
    let entry_point_instance = EntryPoint::new(entry_point, provider);
    let nonce = entry_point_instance.getNonce(sender, key).call().await?;
    Ok(nonce)
}

pub fn get_nonce_call_data(sender: Address, key: U192) -> Bytes {
    let call = EntryPoint::getNonceCall { sender, key };
    call.abi_encode().into()
}
