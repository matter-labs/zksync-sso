use sdk::api::account::deployment::DeployedAccountDetails as SdkDeployedAccountDetails;

mod balance;
mod debug;
mod deployment;
mod fetch;
mod modular_account;
mod owners;
mod passkey;
mod send;
mod session;
mod transaction;
mod validators;

#[derive(Debug, Clone, uniffi::Record)]
pub struct Account {
    pub address: String,
    pub unique_account_id: String,
}

impl From<SdkDeployedAccountDetails> for Account {
    fn from(deployed_account_details: SdkDeployedAccountDetails) -> Self {
        Self {
            address: deployed_account_details.address.to_string(),
            unique_account_id: deployed_account_details.unique_account_id,
        }
    }
}
