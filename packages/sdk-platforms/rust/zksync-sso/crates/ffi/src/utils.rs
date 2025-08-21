use sdk::api::utils::private_key_to_address as sdk_private_key_to_address;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum PrivateKeyToAddressError {
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
}

#[uniffi::export]
pub fn private_key_to_address(
    private_key: String,
) -> Result<String, PrivateKeyToAddressError> {
    sdk_private_key_to_address(&private_key)
        .map(|address| address.to_string())
        .map_err(|e| PrivateKeyToAddressError::InvalidPrivateKey(e.to_string()))
}
