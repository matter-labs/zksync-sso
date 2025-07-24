use alloy::{
    network::TransactionBuilder,
    primitives::{Address, U256},
};
use alloy_zksync::network::transaction_request::TransactionRequest;
use std::fmt::Debug;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Transaction {
    pub from: Address,
    pub to: Option<Address>,
    pub value: Option<String>,
    pub input: Option<String>,
}

impl TryFrom<Transaction> for TransactionRequest {
    type Error = eyre::Error;

    fn try_from(transaction: Transaction) -> Result<Self, Self::Error> {
        let mut transaction_request =
            TransactionRequest::default().with_from(transaction.from);
        if let Some(to) = transaction.to {
            transaction_request = transaction_request.with_to(to);
        }
        if let Some(value) = transaction.value {
            transaction_request =
                transaction_request.with_value(value.parse::<U256>()?);
        }
        if let Some(input) = transaction.input {
            let input_bytes = alloy::hex::decode(input)?;
            transaction_request = transaction_request.with_input(input_bytes);
        }
        Ok(transaction_request)
    }
}
