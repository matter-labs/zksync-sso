use crate::config;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FundAccountError {
    #[error("{0}")]
    FundAccount(String),
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn fund_account(
    address: String,
    config: config::Config,
) -> Result<(), FundAccountError> {
    let money = sdk::api::account::fund::generate_random_eth();
    sdk::api::account::fund::fund_account(
        address.parse().expect("Invalid address"),
        money.minor_value(),
        &(config.try_into()
            as Result<sdk::config::Config, config::ConfigError>)
            .map_err(|e: config::ConfigError| {
                FundAccountError::FundAccount(e.to_string())
            })?,
    )
    .await
    .map_err(|e| FundAccountError::FundAccount(e.to_string()))
}
