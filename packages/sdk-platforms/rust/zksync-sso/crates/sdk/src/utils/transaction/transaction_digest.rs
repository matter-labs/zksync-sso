use crate::utils::transaction::Transaction;
use alloy::{
    dyn_abi::TypedData,
    network::TransactionBuilder,
    primitives::{FixedBytes, hex},
    sol_types::{Eip712Domain, SolStruct, eip712_domain},
};
use alloy_zksync::network::transaction_request::TransactionRequest;
use log::debug;
use serde_json::{self};

#[derive(Debug, PartialEq, Eq)]
pub struct TransactionDigest(pub FixedBytes<32>);

pub(crate) fn create_domain(chain_id: u64) -> Eip712Domain {
    eip712_domain! {
        name: "zkSync",
        version: "2",
        chain_id: chain_id,
    }
}

pub fn get_digest(
    transaction: TransactionRequest,
) -> eyre::Result<TransactionDigest> {
    let chain_id =
        transaction.chain_id().ok_or(eyre::eyre!("Chain ID is required"))?;
    let sol_transaction: Transaction = transaction.try_into()?;
    let domain = create_domain(chain_id);

    let typed_data =
        TypedData::from_struct(&sol_transaction, Some(domain.clone()));
    debug!(
        "XDB - get_transaction_digest_from_sol - TypedData: {}",
        serde_json::to_string_pretty(&typed_data)?
    );

    let digest = sol_transaction.eip712_signing_hash(&domain);
    debug!(
        "XDB - get_transaction_digest_from_sol - digest: {}",
        digest.clone()
    );
    debug!(
        "XDB - get_transaction_digest_from_sol - hex::encode(digest): {}",
        hex::encode(digest)
    );

    Ok(TransactionDigest(digest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Bytes, U256, address, hex};

    #[test]
    fn test_get_digest() {
        // Arrange
        let mock_address = address!("1234567890123456789012345678901234567890");
        let mock_to = address!("0987654321098765432109876543210987654321");

        let mut tx_request = TransactionRequest::default();
        tx_request.set_from(mock_address);
        tx_request.set_to(mock_to);
        tx_request.set_gas_limit(2000000);
        tx_request.set_gas_per_pubdata(U256::from(50000));
        tx_request.set_max_fee_per_gas(0);
        tx_request.set_max_priority_fee_per_gas(0);
        tx_request.set_nonce(1);
        tx_request.set_value(U256::from(1000000000000000000u128));
        tx_request.set_input(Bytes::from(Vec::new()));
        tx_request.set_factory_deps(vec![]);
        tx_request.set_chain_id(280);

        let expected_digest = "0x749a8ecc71bc5af1593e9b2a3cc2d22f624bf9b2787cf224cf171ad7ba94caa5";

        // Act
        let result = get_digest(tx_request).unwrap();

        // Assert
        let hash_hex = format!("0x{}", hex::encode(result.0));

        println!("\n=== Transaction Digest Test Results ===");
        println!("Hash: {hash_hex}");
        println!("Expected: {expected_digest}");

        assert_eq!(hash_hex, expected_digest);
    }
}
