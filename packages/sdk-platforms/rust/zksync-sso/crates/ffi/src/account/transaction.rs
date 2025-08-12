use sdk::api::utils::parse_address;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum TransactionConversionError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
}

#[derive(Debug, uniffi::Record)]
pub struct Transaction {
    pub from: String,
    pub to: Option<String>,
    pub value: Option<String>,
    pub input: Option<String>,
}

impl TryFrom<Transaction> for sdk::api::account::transaction::Transaction {
    type Error = TransactionConversionError;

    fn try_from(tx: Transaction) -> Result<Self, Self::Error> {
        Ok(Self {
            from: parse_address(&tx.from).map_err(|e| {
                TransactionConversionError::InvalidAddress(e.to_string())
            })?,
            to: tx.to.and_then(|to| {
                parse_address(&to)
                    .map_err(|e| {
                        TransactionConversionError::InvalidAddress(
                            e.to_string(),
                        )
                    })
                    .ok()
            }),
            value: tx.value,
            input: tx.input,
        })
    }
}
