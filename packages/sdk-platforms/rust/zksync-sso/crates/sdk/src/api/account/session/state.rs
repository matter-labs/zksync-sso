use crate::{
    client::session::actions::session::state::{
        GetSessionStateArgs as GetSessionStateArgsAction,
        GetSessionStateReturnType as GetSessionStateReturnTypeAction,
        get_session_state as get_session_state_action,
    },
    config::Config,
    utils::session::session_lib::{
        session_spec::SessionSpec, session_state::SessionState,
    },
};
use alloy::primitives::Address;
use std::fmt::Debug;

/// Arguments for revoking a session
#[derive(Debug, Clone)]
pub struct GetSessionStateArgs {
    /// Account address
    pub account: Address,
    /// Session configuration
    pub session_config: SessionSpec,
}

impl From<GetSessionStateArgs> for GetSessionStateArgsAction {
    fn from(val: GetSessionStateArgs) -> Self {
        GetSessionStateArgsAction {
            account: val.account,
            session_config: val.session_config,
        }
    }
}

/// Return type for session creation
#[derive(Debug, Clone)]
pub struct GetSessionStateReturnType {
    /// Session state
    pub session_state: SessionState,
}

impl From<GetSessionStateReturnTypeAction> for GetSessionStateReturnType {
    fn from(value: GetSessionStateReturnTypeAction) -> Self {
        GetSessionStateReturnType { session_state: value.session_state }
    }
}

pub async fn get_session_state(
    args: GetSessionStateArgs,
    config: &Config,
) -> eyre::Result<GetSessionStateReturnType> {
    get_session_state_action(args.into(), config).await.map(Into::into)
}
