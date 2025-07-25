use crate::{
    client::session::actions::session::hash::{
        SessionHash, get_session_hash as get_session_hash_client,
    },
    utils::session::session_lib::session_spec::SessionSpec,
};

pub fn get_session_hash(
    session_config: SessionSpec,
) -> eyre::Result<SessionHash> {
    get_session_hash_client(session_config)
}
