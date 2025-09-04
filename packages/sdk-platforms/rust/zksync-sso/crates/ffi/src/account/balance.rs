use crate::config;
use sdk::api::{
    account::balance::{
        GetAccountBalanceResult as SdkGetAccountBalanceResult,
        get_balance as sdk_get_balance,
    },
    utils::parse_address,
};

#[derive(Debug, uniffi::Record)]
pub struct AccountBalance {
    pub balance: String,
}

impl From<SdkGetAccountBalanceResult> for AccountBalance {
    fn from(result: SdkGetAccountBalanceResult) -> Self {
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
    let address = parse_address(&address).map_err(|_| {
        GetAccountBalanceError::GetBalance("Invalid address".to_string())
    })?;
    let sdk_config = config.try_into().map_err(|e: config::ConfigError| {
        GetAccountBalanceError::GetBalance(e.to_string())
    })?;
    sdk_get_balance(address, &sdk_config)
        .await
        .map_err(|e| GetAccountBalanceError::GetBalance(e.to_string()))
        .map(Into::into)
}
