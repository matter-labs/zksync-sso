mod balance;
mod deployment;
mod fetch;
mod fund;
mod send;
mod session;
mod transaction;

#[derive(Debug, Clone, uniffi::Record)]
pub struct Account {
    pub address: String,
    pub unique_account_id: String,
}
