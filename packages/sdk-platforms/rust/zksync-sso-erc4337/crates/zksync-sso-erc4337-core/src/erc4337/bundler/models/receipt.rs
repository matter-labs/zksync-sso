use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationReceiptReceipt {
    pub transaction_hash: String,
    pub transaction_index: String,
    pub block_hash: String,
    pub block_number: String,
    pub from: Address,
    pub to: Address,
    pub cumulative_gas_used: String,
    pub gas_used: String,
    pub contract_address: Option<String>,
    pub status: String,
    pub logs_bloom: String,
    // pub r#type: String,
    pub effective_gas_price: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationReceipt {
    pub user_op_hash: String,
    pub entry_point: Address,
    pub sender: Address,
    pub nonce: String,
    pub paymaster: String,
    pub actual_gas_cost: String,
    pub actual_gas_used: String,
    pub success: bool,
    // pub reason: String,
    pub receipt: UserOperationReceiptReceipt,
    // TODO: add `logs` property
}

// {
//   "jsonrpc": "2.0",
//   "id": 1,
//   "result": {
//         "entryPoint": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
//         "userOpHash": "0xa5a579c6fd86c2d8a4d27f5bb22796614d3a31bbccaba8f3019ec001e001b95f",
//         "sender": "0x8C6bdb488F664EB98E12cB2671fE2389Cc227D33",
//         "nonce": "0x18554d9a95404c5e8ac591f8608a18f80000000000000000",
//         "actualGasUsed": "0x7f550",
//         "actualGasCost": "0x4b3b147f788710",
//         "success": true,
//         "logs": [
//             // ...
//         ],
//         "receipt": {
//             "transactionHash": "0x57465d20d634421008a167cfcfcde94847dba9d6b5d3652b071d4b84e5ce74ff",
//             "transactionIndex": "0x20",
//             "blockHash": "0xeaeec1eff4095bdcae44d86574cf1bf08b14b26be571b7c2290f32f9f250c103",
//             "blockNumber": "0x31de70e",
//             "from": "0x43370996A3Aff7B66B3AC7676dD973d01Ecec039",
//             "to": "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789",
//             "cumulativeGasUsed": "0x4724c3",
//             "gasUsed": "0x7ff5a",
//             "address": null,
//             "logs": [
//                 // ...
//             ],
//             "logsBloom": "0x010004000800020000000040000000000000040000000000000010000004000000080000001000000212841100000000041080000000000020000240000000000800000022001000400000080000028000040000000000200001000010000000000000000a0000000000000000800800000000004110004080800110282000000000000002000000000000000000000000000200000400000000000000240040200002000000000000400000000002000140000000000000000002200000004000000002000000000021000000000000000000000000800080108020000020000000080000000000000000000000000000000000000000000108000000102000",
//             "status": "0x1",
//             "effectiveGasPrice": "0x89b098f46"
//         }
//     }
// }

impl UserOperationReceipt {
    pub fn mock() -> Self {
        UserOperationReceipt {
            user_op_hash: "0x93c06f3f5909cc2b192713ed9bf93e3e1fde4b22fcd2466304fa404f9b80ff90".to_string(),
            entry_point: "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
                .parse()
                .unwrap(),
            sender: "0x9E1276a4A64D064256E7347cdA4d8C8039b1bc48".parse().unwrap(),
            nonce: "0x3".to_string(),
            paymaster: "0xb80bCD1Bcf735238EAB64ffc3431076605A21D61".to_string(),
            actual_gas_cost: "0x11bed797b2d5c8".to_string(),
            actual_gas_used: "0x20725".to_string(),
            success: true,
            // reason: "".to_string(),
            receipt: UserOperationReceiptReceipt {
                transaction_hash: "0x68b5465c1efe05e5a29f8551c3808e5fd3b0a46e7abb007e11c586632cf46c23".to_string(),
                transaction_index: "0x85".to_string(),
                block_hash: "0x0b95eb450c36397458e77e38420b89f0b6336b7c61b7bbb9898e0318da0f4cd0".to_string(),
                block_number: "0x113fc81".to_string(),
                from: "0x374a2c4dcb38ecbb606117ae1bfe402a52176ec1".parse().unwrap(),
                to: "0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789".parse().unwrap(),
                cumulative_gas_used: "0x12bafe6".to_string(),
                gas_used: "0x20d07".to_string(),
                contract_address: None,
                status: "0x1".to_string(),
                logs_bloom: "0x04400000000040002000000000000000000000000000000000000000000000000008000000000000000200010000000000100000000000000000020000000000000000000000000000000008000000000100000000000000000000000000000000000000080000000008000000000000000000000000000000000010000000000000000000040040100088000000000000000000000000000000000000000000000000000000000100400000000008000000000000000000000002000000000000000002000000100001000000000000000000002000000000000040000000000000000000000000200000000000000000000000000000000000000000000010".to_string(),
                // r#type: "0x2".to_string(),
                effective_gas_price: "0x86cb70a28".to_string(),
            },
        }
    }
}
