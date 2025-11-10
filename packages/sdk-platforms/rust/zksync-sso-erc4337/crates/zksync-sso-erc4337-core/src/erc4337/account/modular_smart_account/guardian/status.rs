use crate::erc4337::account::modular_smart_account::guardian::contract::GuardianExecutor::{self, guardianStatusForReturn};
use alloy::{
    primitives::Address,
    providers::Provider,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuardianStatus {
    /// The guardian does not exist for the account.
    DoesNotExist,
    /// The guardian exists for the account but is not active.
    PresentNotActive,
    /// The guardian exists for the account and is active.
    Active,
}

impl GuardianStatus {
    pub fn is_present_but_not_active(&self) -> bool {
        self == &GuardianStatus::PresentNotActive
    }

    pub fn is_active(&self) -> bool {
        self == &GuardianStatus::Active
    }
}

impl From<guardianStatusForReturn> for GuardianStatus {
    fn from(status: guardianStatusForReturn) -> Self {
        match (status.isPresent, status.isActive) {
            (true, true) => GuardianStatus::Active,
            (true, false) => GuardianStatus::PresentNotActive,
            (false, _) => GuardianStatus::DoesNotExist,
        }
    }
}

pub async fn get_guardian_status<P>(
    account_address: Address,
    guardian_address: Address,
    guardian_executor_address: Address,
    provider: P,
) -> eyre::Result<GuardianStatus>
where
    P: Provider + Send + Sync + Clone,
{
    let guardian_executor =
        GuardianExecutor::new(guardian_executor_address, provider);
    let status = guardian_executor
        .guardianStatusFor(account_address, guardian_address)
        .call()
        .await?;
    Ok(status.into())
}
