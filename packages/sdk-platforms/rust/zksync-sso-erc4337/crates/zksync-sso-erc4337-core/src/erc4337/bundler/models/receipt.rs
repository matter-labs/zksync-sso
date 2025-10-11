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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationReceipt {
    pub user_op_hash: String,
    pub entry_point: Address,
    pub sender: Address,
    pub nonce: String,
    pub actual_gas_cost: String,
    pub actual_gas_used: String,
    pub success: bool,
    pub receipt: UserOperationReceiptReceipt,
    pub logs: Vec<UserOperationReceiptLog>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationReceiptLog {
    pub transaction_index: String,
    pub block_number: String,
    pub transaction_hash: String,
    pub address: Address,
    pub topics: Vec<String>,
    pub data: String,
    pub block_hash: String,
    pub log_index: String,
}

impl UserOperationReceipt {
    pub fn mock() -> Self {
        UserOperationReceipt {
            user_op_hash: "0x93c06f3f5909cc2b192713ed9bf93e3e1fde4b22fcd2466304fa404f9b80ff90".to_string(),
            entry_point: "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
                .parse()
                .unwrap(),
            sender: "0x9E1276a4A64D064256E7347cdA4d8C8039b1bc48".parse().unwrap(),
            nonce: "0x3".to_string(),
            actual_gas_cost: "0x11bed797b2d5c8".to_string(),
            actual_gas_used: "0x20725".to_string(),
            success: true,
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
            },
            logs: vec![],
        }
    }
}
