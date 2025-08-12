use crate::utils::session::session_lib::session_spec::SessionSpec;
pub use crate::utils::session::session_lib::*;

pub fn session_spec_from_json(json: &str) -> eyre::Result<SessionSpec> {
    let session_spec: SessionSpec = serde_json::from_str(json)
        .map_err(|e| eyre::eyre!("Invalid session spec: {}", e))?;
    Ok(session_spec)
}
