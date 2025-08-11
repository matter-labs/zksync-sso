use sdk::api::account::session::{
    decode_session_config, hash::get_session_hash as sdk_get_session_hash,
};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum GetSessionHashError {
    #[error("{0}")]
    GetSessionHash(String),
    #[error("Invalid session config: {0}")]
    InvalidSessionConfig(String),
}

#[uniffi::export]
pub fn get_session_hash(
    session_config_json: String,
) -> Result<String, GetSessionHashError> {
    let session_config =
        decode_session_config(&session_config_json).map_err(|e| {
            GetSessionHashError::InvalidSessionConfig(e.to_string())
        })?;

    let hash = sdk_get_session_hash(session_config)
        .map_err(|e| GetSessionHashError::GetSessionHash(e.to_string()))?;

    Ok(format!("{:#x}", hash.fixed_bytes()))
}
