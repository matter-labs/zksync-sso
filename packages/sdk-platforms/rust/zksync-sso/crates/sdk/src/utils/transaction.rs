use alloy::{
    network::TransactionBuilder,
    primitives::{Address, U256, keccak256},
    sol,
};
use alloy_zksync::network::transaction_request::TransactionRequest;
use serde::{Deserialize, Serialize};

pub mod transaction_digest;

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

        Ok(transaction)
    }
}
