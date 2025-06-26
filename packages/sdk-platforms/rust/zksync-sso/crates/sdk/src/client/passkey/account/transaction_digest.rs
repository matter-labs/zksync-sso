use alloy::{
    dyn_abi::TypedData,
    network::TransactionBuilder,
    primitives::{Address, FixedBytes, U256, hex, keccak256},
    sol,
    sol_types::{Eip712Domain, SolStruct, eip712_domain},
};
use alloy_zksync::network::transaction_request::TransactionRequest;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::{self};

#[derive(Debug, PartialEq, Eq)]
pub struct TransactionDigest(pub FixedBytes<32>);

sol! {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct Transaction {
        uint256 txType;
        uint256 from;
        uint256 to;
        uint256 gasLimit;
        uint256 gasPerPubdataByteLimit;
        uint256 maxFeePerGas;
        uint256 maxPriorityFeePerGas;
        uint256 paymaster;
        uint256 nonce;
        uint256 value;
        bytes data;
        bytes32[] factoryDeps;
        bytes paymasterInput;
    }
}

fn create_domain(chain_id: u64) -> Eip712Domain {
    eip712_domain! {
        name: "zkSync",
        version: "2",
        chain_id: chain_id,
    }
}

impl TryFrom<TransactionRequest> for Transaction {
    type Error = eyre::Error;

    fn try_from(tx: TransactionRequest) -> Result<Self, Self::Error> {
        let from = {
            let from =
                tx.from().ok_or(eyre::eyre!("From address is required"))?;
            let mut from_bytes = [0u8; 32];
            from_bytes[12..].copy_from_slice(from.as_slice());
            U256::from_be_bytes(from_bytes)
        };

        let to = {
            let mut to_bytes = [0u8; 32];
            to_bytes[12..]
                .copy_from_slice(tx.to().unwrap_or(Address::ZERO).as_slice());
            U256::from_be_bytes(to_bytes)
        };

        let factory_deps = tx
            .clone()
            .factory_deps()
            .map(ToOwned::to_owned)
            .unwrap_or_default()
            .into_iter()
            .map(|dep| keccak256(&dep))
            .collect();

        let paymaster_params =
            tx.paymaster_params().map(ToOwned::to_owned).unwrap_or_default();

        let paymaster = {
            let paymaster = paymaster_params.paymaster;
            let mut paymaster_address_bytes = [0u8; 32];
            paymaster_address_bytes[12..].copy_from_slice(paymaster.as_slice());
            U256::from_be_bytes(paymaster_address_bytes)
        };

        let paymaster_input = paymaster_params.paymaster_input;

        let data = tx.input().to_owned().unwrap_or_default().to_owned();

        let transaction = Transaction {
            txType: U256::from(113),
            from,
            to,
            gasLimit: U256::from(tx.gas_limit().unwrap_or(0)),
            gasPerPubdataByteLimit: tx
                .gas_per_pubdata()
                .unwrap_or(U256::from(50000)),
            maxFeePerGas: U256::from(tx.max_fee_per_gas().unwrap_or(0)),
            maxPriorityFeePerGas: U256::from(
                tx.max_priority_fee_per_gas().unwrap_or(0),
            ),
            paymaster,
            nonce: U256::from(tx.nonce().unwrap_or(0)),
            value: tx.value().unwrap_or(U256::ZERO),
            data,
            factoryDeps: factory_deps,
            paymasterInput: paymaster_input,
        };

        debug!(
            "XDB - transform_to_transaction - raw data: {}",
            hex::encode(&transaction.data)
        );
        debug!(
            "XDB - transform_to_transaction - raw paymasterInput: {}",
            hex::encode(&transaction.paymasterInput)
        );

        debug!(
            "XDB - transform_to_transaction - raw factoryDeps: {:?}",
            transaction.factoryDeps.iter().map(hex::encode).collect::<Vec<_>>()
        );

        Ok(transaction)
    }
}

pub fn get_digest(
    transaction: TransactionRequest,
) -> Result<TransactionDigest, Box<dyn std::error::Error>> {
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
        println!("Hash: {}", hash_hex);
        println!("Expected: {}", expected_digest);

        assert_eq!(hash_hex, expected_digest);
    }
}
