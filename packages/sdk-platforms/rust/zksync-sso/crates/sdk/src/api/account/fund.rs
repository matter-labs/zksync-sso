use crate::{
    config::Config,
    utils::{
        alloy::extensions::ProviderExt, anvil_zksync::rich_wallet::RichWallet,
    },
};
use alloy::{
    network::TransactionBuilder,
    primitives::{Address, U256},
    providers::Provider,
};
use alloy_zksync::{
    network::transaction_request::TransactionRequest, provider::zksync_provider,
};
use log::debug;
use money::Money;
use rand::Rng;

// TODO: move this to testing module
pub async fn fund_account(
    address: Address,
    amount: U256,
    config: &Config,
) -> eyre::Result<()> {
    debug!("XDB fund_account - address: {address:?}");

    let provider = {
        let wallet = RichWallet::two().to_zksync_wallet()?;
        zksync_provider()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(config.node_url.clone())
    };

    let tx = TransactionRequest::default().with_to(address).with_value(amount);

    let receipt = provider.send_transaction(tx).await?;

    let tx_hash = receipt.tx_hash().to_owned();

    let receipt = provider.wait_for_transaction_receipt(tx_hash).await?;

    debug!("XDB fund_account - Got receipt: {receipt:#?}");

    Ok(())
}

pub fn generate_random_eth() -> Money {
    let mut rng = rand::rng();

    let whole_eth = rng.random_range(0..=10);

    let decimals = rng.random_range(0..=999_999_999_999_999_999u64);

    let wei = U256::from(whole_eth)
        .checked_mul(U256::from(10).pow(U256::from(18)))
        .unwrap()
        .checked_add(U256::from(decimals))
        .unwrap();

    debug!(
        "XDB - generate_random_eth - Generated {whole_eth}.{decimals:018} ETH ({wei} wei)"
    );

    Money::eth(wei)
}
