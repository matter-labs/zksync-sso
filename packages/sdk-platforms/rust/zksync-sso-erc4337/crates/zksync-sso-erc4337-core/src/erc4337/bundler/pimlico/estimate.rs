// {
//   "jsonrpc": "2.0",
//   "id": 1,
//   "result": {
//     "preVerificationGas": "0xd3e3",
//     "verificationGasLimit": "0x60b01",
//     "callGasLimit": "0x13880",
//     "paymasterVerificationGasLimit": "0x0",
//     "paymasterPostOpGasLimit": "0x0"
//   }
// }

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
