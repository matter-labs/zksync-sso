#[cfg(test)]
pub mod test_utilities;

use alloy::{network::EthereumWallet, signers::local::PrivateKeySigner};
use std::str::FromStr;

pub fn ethereum_wallet_from_private_key(
    signer_private_key: &str,
) -> eyre::Result<EthereumWallet> {
    let signer = PrivateKeySigner::from_str(signer_private_key)?;
    let ethereum_wallet = EthereumWallet::new(signer);
    Ok(ethereum_wallet)
}
