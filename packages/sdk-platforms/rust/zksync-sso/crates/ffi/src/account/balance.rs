use crate::config;

#[derive(Debug, uniffi::Record)]
pub struct AccountBalance {
    pub balance: String,
}

impl From<sdk::api::account::balance::GetAccountBalanceResult>
    for AccountBalance
{
    fn from(
        result: sdk::api::account::balance::GetAccountBalanceResult,
    ) -> Self {
        Self { balance: result.balance }
    }
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum GetAccountBalanceError {
    #[error("{0}")]
    GetBalance(String),
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn get_balance(
    address: String,
    config: config::Config,
) -> Result<AccountBalance, GetAccountBalanceError> {
    let address = sdk::utils::alloy::parse_address(&address).map_err(|_| {
        GetAccountBalanceError::GetBalance("Invalid address".to_string())
    })?;
    sdk::api::account::balance::get_balance(
        address,
        &(config.try_into()
            as Result<sdk::config::Config, config::ConfigError>)
            .map_err(|e: config::ConfigError| {
                GetAccountBalanceError::GetBalance(e.to_string())
            })?,
    )
    .await
    .map_err(|e| GetAccountBalanceError::GetBalance(e.to_string()))
    .map(Into::into)
}
