use crate::utils::create_error;
use alloy::eips::eip2718::Encodable2718;
use alloy::network::{EthereumWallet, TransactionBuilder};
use alloy::primitives::{Address, FixedBytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Deserialize, Debug)]
pub struct TxParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub value: Option<String>,
    pub data: Option<String>,
    pub gas: Option<String>,
    pub gas_price: Option<String>,
    pub nonce: Option<String>,
}

#[derive(Serialize)]
pub struct SignedTxResult {
    pub hash: String,
    pub encoded: String,
    pub signature: Option<String>,
}

#[derive(Serialize)]
pub struct FilledTxResult {
    pub from: Option<String>,
    pub to: Option<String>,
    pub value: Option<String>,
    pub gas: Option<String>,
    pub gas_price: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub nonce: Option<String>,
    pub data: Option<String>,
    pub status: String,
}

#[derive(Serialize)]
pub struct SendTxResult {
    pub tx_hash: String,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub gas_used: String,
    pub effective_gas_price: String,
    pub block_number: Option<String>,
    pub status: String,
}

pub fn params_to_transaction_request(params: TxParams) -> Result<TransactionRequest, String> {
    let mut tx = TransactionRequest::default();

    if let Some(from) = params.from {
        let from_addr = from
            .parse::<Address>()
            .map_err(|e| create_error(&e.to_string()))?;
        tx = tx.with_from(from_addr);
    }

    if let Some(to) = params.to {
        tx = tx.with_to(
            to.parse::<Address>()
                .map_err(|e| create_error(&e.to_string()))?,
        );
    }

    if let Some(value) = params.value {
        tx = tx.with_value(
            value
                .parse::<U256>()
                .map_err(|e| create_error(&e.to_string()))?,
        );
    }

    if let Some(data) = params.data {
        let bytes = alloy::primitives::hex::decode(data.trim_start_matches("0x"))
            .map_err(|e| create_error(&e.to_string()))?;
        tx = tx.with_input(bytes);
    }

    if let Some(gas) = params.gas {
        tx = tx.with_gas_limit(
            gas.parse::<u64>()
                .map_err(|e| create_error(&e.to_string()))?,
        );
    }

    if let Some(gas_price) = params.gas_price {
        tx = tx.with_gas_price(
            gas_price
                .parse::<u128>()
                .map_err(|e| create_error(&e.to_string()))?,
        );
    }

    if let Some(nonce) = params.nonce {
        tx = tx.with_nonce(
            nonce
                .parse::<u64>()
                .map_err(|e| create_error(&e.to_string()))?,
        );
    }

    Ok(tx)
}

pub async fn create_transaction_request(params: TxParams) -> Result<SignedTxResult, String> {
    debug!("Received params: {:?}", params);

    let dummy_private_key = FixedBytes::<32>::from([1u8; 32]);
    let signer = PrivateKeySigner::from_bytes(&dummy_private_key)
        .map_err(|e| create_error(&e.to_string()))?;
    let wallet = EthereumWallet::from(signer);

    let mut tx = params_to_transaction_request(params)?;

    if tx.nonce.is_none() {
        tx = tx.with_nonce(0);
    }
    if tx.gas_price.is_none() && tx.max_fee_per_gas.is_none() {
        tx = tx.with_max_fee_per_gas(20_000_000_000u128);
        tx = tx.with_max_priority_fee_per_gas(1_000_000_000u128);
    }

    let built_tx = tx
        .build(&wallet)
        .await
        .map_err(|e| create_error(&format!("Failed to build transaction: {e}")))?;

    debug!("Successfully built and signed transaction!");
    let tx_hash = built_tx.tx_hash();
    let encoded_tx = built_tx.encoded_2718();

    Ok(SignedTxResult {
        hash: format!("0x{}", alloy::primitives::hex::encode(tx_hash.as_slice())),
        encoded: format!("0x{}", alloy::primitives::hex::encode(&encoded_tx)),
        signature: None,
    })
}

pub async fn fill_transaction(params: TxParams) -> Result<FilledTxResult, String> {
    let url = "http://127.0.0.1:8545".parse().unwrap();
    let provider = ProviderBuilder::new().connect_http(url);

    let tx = params_to_transaction_request(params)?;

    let _filled_tx = provider
        .fill(tx.clone())
        .await
        .map_err(|e| create_error(&format!("Failed to fill transaction: {e}")))?;

    Ok(FilledTxResult {
        from: tx
            .from
            .map(|addr| format!("0x{}", alloy::primitives::hex::encode(addr.as_slice()))),
        to: match &tx.to {
            Some(alloy::primitives::TxKind::Call(addr)) => Some(format!(
                "0x{}",
                alloy::primitives::hex::encode(addr.as_slice())
            )),
            Some(alloy::primitives::TxKind::Create) => Some("CREATE".to_string()),
            None => None,
        },
        value: tx.value.map(|v| v.to_string()),
        gas: tx.gas.map(|g| g.to_string()),
        gas_price: tx.gas_price.map(|gp| gp.to_string()),
        max_fee_per_gas: tx.max_fee_per_gas.map(|mfpg| mfpg.to_string()),
        max_priority_fee_per_gas: tx.max_priority_fee_per_gas.map(|mpfpg| mpfpg.to_string()),
        nonce: tx.nonce.map(|n| n.to_string()),
        data: tx
            .input
            .input
            .as_ref()
            .map(|d| format!("0x{}", alloy::primitives::hex::encode(d))),
        status: "filled".to_string(),
    })
}

pub async fn send_transaction(params: TxParams) -> Result<SendTxResult, String> {
    let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let key_bytes =
        alloy::primitives::hex::decode(private_key).map_err(|e| create_error(&e.to_string()))?;

    let signer = PrivateKeySigner::from_bytes(
        &FixedBytes::<32>::try_from(key_bytes.as_slice())
            .map_err(|e| create_error(&format!("Invalid key length: {e}")))?,
    )
    .map_err(|e| create_error(&e.to_string()))?;

    let wallet = EthereumWallet::from(signer);

    let url = "http://127.0.0.1:8545".parse().unwrap();
    let provider = ProviderBuilder::new().wallet(wallet).connect_http(url);

    let tx = params_to_transaction_request(params)?;

    let pending_tx = provider
        .send_transaction(tx)
        .await
        .map_err(|e| create_error(&format!("Failed to send transaction: {e}")))?;

    let tx_hash = *pending_tx.tx_hash();

    let receipt = pending_tx
        .get_receipt()
        .await
        .map_err(|e| create_error(&format!("Failed to get transaction receipt: {e}")))?;

    Ok(SendTxResult {
        tx_hash: format!("0x{}", alloy::primitives::hex::encode(tx_hash.as_slice())),
        from: format!(
            "0x{}",
            alloy::primitives::hex::encode(receipt.from.as_slice())
        ),
        to: receipt
            .to
            .map(|addr| format!("0x{}", alloy::primitives::hex::encode(addr.as_slice()))),
        value: "0".to_string(),
        gas_used: receipt.gas_used.to_string(),
        effective_gas_price: receipt.effective_gas_price.to_string(),
        block_number: receipt.block_number.map(|n| n.to_string()),
        status: if receipt.status() {
            "success"
        } else {
            "failed"
        }
        .to_string(),
    })
}
