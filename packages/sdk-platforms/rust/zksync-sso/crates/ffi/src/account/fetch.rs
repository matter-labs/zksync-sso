use crate::config;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FetchAccountError {
    #[error("{0}")]
    FetchAccount(String),
}

impl From<sdk::api::account::Account> for super::Account {
    fn from(value: sdk::api::account::Account) -> Self {
        Self {
            address: value.address.to_string(),
            unique_account_id: value.unique_account_id,
        }
    }
}

impl From<sdk::api::account::fetch::FetchedAccount> for super::Account {
    fn from(value: sdk::api::account::fetch::FetchedAccount) -> Self {
        Self {
            address: value.address.to_string(),
            unique_account_id: value.unique_account_id,
        }
    }
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn fetch_account(
    unique_account_id: String,
    expected_origin: String,
    config: config::Config,
) -> Result<super::Account, FetchAccountError> {
    sdk::api::account::fetch::fetch_account(
        unique_account_id,
        expected_origin,
        &(config.try_into()
            as Result<sdk::config::Config, config::ConfigError>)
            .map_err(|e: config::ConfigError| {
                FetchAccountError::FetchAccount(e.to_string())
            })?,
    )
    .await
    .map_err(|e| FetchAccountError::FetchAccount(e.to_string()))
    .map(Into::into)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn get_account_by_user_id(
    unique_account_id: String,
    secret_account_salt: String,
    config: config::Config,
) -> Result<super::Account, FetchAccountError> {
    sdk::api::account::fetch::get_account_by_user_id(
        unique_account_id,
        secret_account_salt,
        &(config.try_into()
            as Result<sdk::config::Config, config::ConfigError>)
            .map_err(|e: config::ConfigError| {
                FetchAccountError::FetchAccount(e.to_string())
            })?,
    )
    .await
    .map_err(|e| FetchAccountError::FetchAccount(e.to_string()))
    .map(Into::into)
}
