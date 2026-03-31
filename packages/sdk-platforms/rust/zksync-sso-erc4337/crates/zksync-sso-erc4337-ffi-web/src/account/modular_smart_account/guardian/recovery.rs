pub mod finalize;
pub mod initialize;
pub mod status;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use zksync_sso_erc4337_core::erc4337::account::modular_smart_account::guardian::recovery::RecoveryType as CoreRecoveryType;

/// Recovery type enum for WASM
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryType {
    None,
    #[allow(clippy::upper_case_acronyms)]
    EOA,
    Passkey,
}

impl From<CoreRecoveryType> for RecoveryType {
    fn from(recovery_type: CoreRecoveryType) -> Self {
        match recovery_type {
            CoreRecoveryType::None => RecoveryType::None,
            CoreRecoveryType::EOA => RecoveryType::EOA,
            CoreRecoveryType::Passkey => RecoveryType::Passkey,
        }
    }
}

impl From<RecoveryType> for CoreRecoveryType {
    fn from(recovery_type: RecoveryType) -> Self {
        match recovery_type {
            RecoveryType::None => CoreRecoveryType::None,
            RecoveryType::EOA => CoreRecoveryType::EOA,
            RecoveryType::Passkey => CoreRecoveryType::Passkey,
        }
    }
}
