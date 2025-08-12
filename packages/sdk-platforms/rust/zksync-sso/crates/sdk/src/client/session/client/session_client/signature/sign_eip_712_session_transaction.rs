use crate::{
    client::session::client::session_client::signature::create_eip_712_session_transaction_signature::create_eip_712_session_transaction_signature,
    config::Config,
    utils::{
        manual_build_transaction::transaction_builder::{
            build_raw_tx
        },
        session::session_lib::session_spec::SessionSpec,
    },
};
use alloy::primitives::{Address,  FixedBytes};
use alloy_zksync::network::{
    transaction_request::TransactionRequest,
};
use log::debug;

pub(crate) async fn sign_eip_712_session_transaction(
    transaction: TransactionRequest,
    address: Address,
    session_key: FixedBytes<32>,
    session_config: SessionSpec,
    config: Config,
) -> eyre::Result<Vec<u8>> {
    debug!("sign_eip_712_session_transaction");

    let (signed_tx, _) = create_eip_712_session_transaction_signature(
        transaction,
        address,
        session_key,
        session_config,
        config,
    )
    .await?;

    debug!("sign_eip_712_session_transaction - signed_tx: {signed_tx:?}");

    let serialized_transaction = build_raw_tx(signed_tx)?;

    debug!(
        "sign_eip_712_session_transaction - serialized_transaction: {serialized_transaction:?}"
    );

    Ok(serialized_transaction)
}
