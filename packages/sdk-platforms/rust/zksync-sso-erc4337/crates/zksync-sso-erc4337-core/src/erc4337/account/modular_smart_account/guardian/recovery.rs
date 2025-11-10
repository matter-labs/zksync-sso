pub mod finalize;
pub mod initialize;
pub mod status;

use crate::erc4337::account::modular_smart_account::guardian::contract::GuardianExecutor::RecoveryType as ContractRecoveryType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecoveryType {
    None,
    EOA,
    Passkey,
}

impl From<RecoveryType> for ContractRecoveryType {
    fn from(recovery_type: RecoveryType) -> Self {
        match recovery_type {
            RecoveryType::None => 0.into(),
            RecoveryType::EOA => 1.into(),
            RecoveryType::Passkey => 2.into(),
        }
    }
}

impl From<RecoveryType> for u8 {
    fn from(recovery_type: RecoveryType) -> Self {
        match recovery_type {
            RecoveryType::None => 0,
            RecoveryType::EOA => 1,
            RecoveryType::Passkey => 2,
        }
    }
}
