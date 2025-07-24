use crate::{
    client::session::client::{
        session_client::signature::create_custom_session_signature::{
            CreateCustomSessionSignatureParameters,
            create_custom_session_signature,
        },
    },
    config::Config,
    utils::{
        session::session_lib::session_spec::SessionSpec,
        transaction::{Transaction, transaction_digest::create_domain},
    },
};
use alloy::{
    network::TransactionBuilder,
    primitives::{Address, Bytes, FixedBytes},
    sol_types::SolStruct,
};
use alloy_zksync::{
    network::{
        transaction_request::TransactionRequest,
    },
};
use log::debug;

pub(crate) async fn create_eip_712_session_transaction_signature(
    transaction: TransactionRequest,
    address: Address,
    session_key: FixedBytes<32>,
    session_config: SessionSpec,
    config: Config,
) -> eyre::Result<(TransactionRequest, Bytes)> {
    debug!("create_eip_712_session_transaction_signature");
    debug!("  transaction: {transaction:?}");
    debug!("  address: {address:?}");
    debug!("  session_key: {session_key:?}");
    debug!("  session_config: {session_config:?}");
    debug!("  config: {config:?}");

    let chain_id =
        transaction.chain_id().ok_or(eyre::eyre!("Chain ID is required"))?;

    let mut tx = transaction.clone();
    tx.set_from(address);

    debug!(
        "create_eip_712_session_transaction_signature - \n\ttx:\n\t\t{tx:?}"
    );

    debug!(
        "create_eip_712_session_transaction_signature - \n\ttx.custom_signature():\n\t\t{:?}",
        tx.custom_signature()
    );

    eyre::ensure!(
        tx.custom_signature().is_some(),
        "Transaction custom signature is required"
    );

    let domain = create_domain(chain_id);
    let sol_transaction: Transaction = tx.clone().try_into()?;
    let digest = sol_transaction.eip712_signing_hash(&domain);
    debug!(
        "create_eip_712_session_transaction_signature - \n\tdigest: {digest:?}"
    );

    let to = transaction.to().ok_or(eyre::eyre!("To address is required"))?;
    debug!("create_eip_712_session_transaction_signature - \n\tto: {to:?}");

    let call_data = transaction.input().map(ToOwned::to_owned);
    debug!(
        "create_eip_712_session_transaction_signature - \n\tcall_data: {call_data:?}"
    );

    let custom_signature = create_custom_session_signature(
        CreateCustomSessionSignatureParameters {
            chain: chain_id,
            session_key,
            config,
            session_config: session_config.clone(),
            hash: digest,
            to,
            call_data,
        },
    )
    .await?;

    debug!(
        "create_eip_712_session_transaction_signature - \n\tcustom_signature: {custom_signature:?}"
    );
    tx.set_custom_signature(custom_signature.clone());

    Ok((tx, custom_signature))
}
