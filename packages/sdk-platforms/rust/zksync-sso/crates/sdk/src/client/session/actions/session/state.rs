use crate::{
    config::Config,
    contracts::SessionKeyValidator,
    utils::session::session_lib::{
        session_spec::SessionSpec, session_state::SessionState,
    },
};
use alloy::primitives::Address;
use alloy_zksync::provider::zksync_provider;
use std::fmt::Debug;
use url;

/// Arguments for revoking a session
#[derive(Debug, Clone)]
pub struct GetSessionStateArgs {
    /// Account address
    pub account: Address,
    /// Session configuration
    pub session_config: SessionSpec,
}

/// Return type for session creation
#[derive(Debug, Clone)]
pub struct GetSessionStateReturnType {
    /// Session state
    pub session_state: SessionState,
}

pub async fn get_session_state(
    args: GetSessionStateArgs,
    config: &Config,
) -> eyre::Result<GetSessionStateReturnType> {
    let provider = {
        let node_url: url::Url = config.clone().node_url;
        zksync_provider().with_recommended_fillers().on_http(node_url.clone())
    };

    let session_validator =
        SessionKeyValidator::new(config.contracts.session, &provider);
    let call_builder = session_validator
        .sessionState(args.account, args.session_config.into());

    let session_state = call_builder.call().await?._0.into();

    Ok(GetSessionStateReturnType { session_state })
}
