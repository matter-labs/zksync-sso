use alloy::primitives::Address;

pub mod account_details;
pub mod balance;
pub mod deployment;
pub mod fetch;
pub mod fund;
pub mod send;

#[derive(Debug, Clone)]
pub struct Account {
    pub address: Address,
    pub unique_account_id: String,
}
