mod balance;
mod deployment;
mod fund;
mod send;

#[derive(Debug, Clone, uniffi::Record)]
pub struct Account {
    pub address: String,
    pub unique_account_id: String,
}
