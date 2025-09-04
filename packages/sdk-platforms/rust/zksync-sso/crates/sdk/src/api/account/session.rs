use crate::utils::session::session_lib::session_spec::SessionSpec;

pub mod client;
pub mod create;
pub mod hash;
pub mod revoke;
pub mod session_lib;
pub mod state;

pub fn decode_session_config(
    session_config_json: &str,
) -> eyre::Result<SessionSpec> {
    let session_config: SessionSpec =
        serde_json::from_str(session_config_json)?;
    Ok(session_config)
}
