use crate::config::Config;
use crate::utils::alloy::extensions::ProviderExt;
use alloy::{
    network::TransactionBuilder,
    primitives::{Address, U256},
    providers::Provider,
};
use alloy_zksync::network::transaction_request::TransactionRequest;
use money::Money;
use rand::Rng;

pub async fn fund_account(
    address: Address,
    amount: U256,
    config: &Config,
) -> eyre::Result<()> {
    println!("XDB fund_account - address: {:?}", address);

    let provider = {
        use alloy::signers::local::PrivateKeySigner;
        use alloy_zksync::{provider::zksync_provider, wallet::ZksyncWallet};
        pub const RICH_WALLET_PRIVATE_KEY_2: &str =
        "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a";

        fn zksync_wallet_3() -> eyre::Result<ZksyncWallet> {
            let signer =
                RICH_WALLET_PRIVATE_KEY_2.parse::<PrivateKeySigner>()?;
            let zksync_wallet = ZksyncWallet::from(signer);
            Ok(zksync_wallet)
        }

        let wallet = zksync_wallet_3()?;

        zksync_provider()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(config.node_url.clone())
    };

    let tx = TransactionRequest::default().with_to(address).with_value(amount);

    let receipt = provider.send_transaction(tx).await?;

    let tx_hash = receipt.tx_hash().to_owned();

    let receipt = provider.wait_for_transaction_receipt(tx_hash).await?;

    println!("XDB fund_account - Got receipt: {receipt:#?}");

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

    println!(
        "XDB - generate_random_eth - Generated {}.{:018} ETH ({} wei)",
        whole_eth, decimals, wei
    );

    Money::eth(wei)
}
