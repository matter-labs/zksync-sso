use alloy::primitives::Address;

pub mod account_details;
pub mod balance;
pub mod deployment;
pub mod fetch;
pub mod fund;
pub mod modular_account;
pub mod owners;
pub mod passkey;
pub mod send;
pub mod session;
pub mod transaction;
pub mod validators;

#[derive(Debug, Clone)]
pub struct Account {
    pub address: Address,
    pub unique_account_id: String,
}
