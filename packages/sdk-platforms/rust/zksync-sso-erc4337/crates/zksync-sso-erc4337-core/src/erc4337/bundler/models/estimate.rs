use alloy::primitives::U256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    pub preVerificationGas: U256,
    pub verificationGasLimit: U256,
    pub callGasLimit: U256,
    pub paymasterVerificationGasLimit: U256,
    pub paymasterPostOpGasLimit: U256,
}
