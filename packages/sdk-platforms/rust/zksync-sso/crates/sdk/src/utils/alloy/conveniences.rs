use crate::config::Config;
use alloy::{providers::Provider, signers::local::PrivateKeySigner};
use alloy_zksync::{
    network::Zksync, provider::zksync_provider, wallet::ZksyncWallet,
};
use std::fmt::Debug;

pub fn get_provider(
    config: &Config,
) -> eyre::Result<impl Provider<Zksync> + Clone + Debug> {
    pub const RICH_WALLET_PRIVATE_KEY_2: &str =
        "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a";

    fn zksync_wallet_3() -> eyre::Result<ZksyncWallet> {
        let signer = RICH_WALLET_PRIVATE_KEY_2.parse::<PrivateKeySigner>()?;
        let zksync_wallet = ZksyncWallet::from(signer);
        Ok(zksync_wallet)
    }

    let wallet = zksync_wallet_3()?;
    let provider =
        zksync_provider().wallet(wallet).on_http(config.node_url.clone());

    Ok(provider)
}
