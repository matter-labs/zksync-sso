use crate::config;
use sdk::api::{
    account::fund::fund_account as sdk_fund_account, utils::u256_from,
};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FundAccountError {
    #[error("{0}")]
    FundAccount(String),
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn fund_account(
    address: String,
    amount: String,
    config: config::Config,
) -> Result<(), FundAccountError> {
    let eth_value = amount.parse::<f64>().map_err(|e| {
        FundAccountError::FundAccount(format!("Invalid ETH amount: {e}"))
    })?;
    let wei = (eth_value * 1e18) as u128;
    let amount = u256_from(wei);
    let sdk_config: sdk::config::Config =
        config.try_into().map_err(|e: config::ConfigError| {
            FundAccountError::FundAccount(e.to_string())
        })?;

    sdk_fund_account(
        address.parse().expect("Invalid address"),
        amount,
        &sdk_config,
    )
    .await
    .map_err(|e| FundAccountError::FundAccount(e.to_string()))
}
