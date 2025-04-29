use std::fmt::Debug;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum PasskeyAuthenticatorError {
    #[error("{0}")]
    Get(String),
}

#[uniffi::export(with_foreign)]
pub trait PasskeyAuthenticator: Send + Sync + Debug {
    fn sign_message(
        &self,
        message: Vec<u8>,
    ) -> Result<Vec<u8>, PasskeyAuthenticatorError>;
}

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait PasskeyAuthenticatorAsync: Send + Sync + Debug {
    async fn sign_message(
        &self,
        message: Vec<u8>,
    ) -> Result<Vec<u8>, PasskeyAuthenticatorError>;
}
