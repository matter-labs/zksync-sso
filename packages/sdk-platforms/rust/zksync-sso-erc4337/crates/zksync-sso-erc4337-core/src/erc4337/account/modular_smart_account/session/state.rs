use crate::erc4337::account::modular_smart_account::session::{
    contract::SessionKeyValidator,
    session_lib::{session_spec::SessionSpec, session_state::SessionState},
};
use alloy::{primitives::Address, providers::Provider};

pub async fn get_session_state<P: Provider + Send + Sync + Clone>(
    account_address: Address,
    session_spec: SessionSpec,
    session_key_validator_address: Address,
    provider: P,
) -> eyre::Result<SessionState> {
    let session_key_validator =
        SessionKeyValidator::new(session_key_validator_address, provider);
    let state = session_key_validator
        .sessionState(account_address, session_spec.into())
        .call()
        .await?;
    Ok(state.into())
}
