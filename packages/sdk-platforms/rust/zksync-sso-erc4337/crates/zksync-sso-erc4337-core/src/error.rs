use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZkSyncSsoError {
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Failed to send user operation: {0}")]
    SendUserOperation(String),

    #[error("User operation validation failed: {0}")]
    UserOperationValidation(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Signature error: {0}")]
    SignatureError(String),
}
