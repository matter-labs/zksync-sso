use crate::{
    client::session::client::{
        SessionTransactionEncodedParamsArgs,
        encoded_session_transaction_signature,
    },
    config::Config,
    utils::session::session_lib::session_spec::SessionSpec,
};
use alloy::{network::TransactionBuilder, primitives::Address};
use alloy_zksync::network::transaction_request::TransactionRequest;
use log::debug;

static EMPTY_SIGNATURE: [u8; 65] = {
    let mut empty_sig_bytes = [0u8; 65];
    empty_sig_bytes[64] = 27u8; // 27u8 (bytes) == 0x1b (Hex)
    empty_sig_bytes
};

pub(crate) fn add_empty_signature_if_not_provided(
    tx_request: TransactionRequest,
    to: Address,
    config: Config,
    session_config: SessionSpec,
) -> eyre::Result<TransactionRequest> {
    if tx_request.custom_signature().is_some() {
        return Ok(tx_request);
    }

    let encoded_empty_signature = {
        let session_key_signed_hash = EMPTY_SIGNATURE.into();
        let session_contract = config.contracts.session;
        let session_config = session_config.clone();
        let call_data = tx_request.input().map(ToOwned::to_owned);
        let args = SessionTransactionEncodedParamsArgs {
            session_key_signed_hash,
            session_contract,
            session_config,
            to,
            call_data,
            timestamp: None,
        };

        let encoded_empty_signature =
            encoded_session_transaction_signature(args)?;

        let encoded_empty_signature_bytes = encoded_empty_signature.to_vec();
        debug!(
            "add_empty_signature_if_not_provided - encoded_empty_signature: {encoded_empty_signature:?}"
        );
        debug!(
            "add_empty_signature_if_not_provided - encoded_empty_signature bytes: {encoded_empty_signature_bytes:?}"
        );

        encoded_empty_signature
    };

    let signed_tx_request =
        tx_request.with_custom_signature(encoded_empty_signature);

    debug!(
        "add_empty_signature_if_not_provided - signed_tx_request: {signed_tx_request:?}"
    );

    Ok(signed_tx_request)
}
