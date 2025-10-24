use crate::erc4337::entry_point::PackedUserOperation;
use alloy::primitives::{Address, B256, Bytes, U256};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestData {
    pub user_operation: UserOperationJson,
    pub metadata: Metadata,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationJson {
    pub sender: String,
    pub nonce: String,
    pub init_code: String,
    pub call_data: String,
    pub call_gas_limit: String,
    pub verification_gas_limit: String,
    pub pre_verification_gas: String,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub paymaster_and_data: String,
    pub signature: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub chain_id: u64,
    pub entry_point_address: String,
    pub entry_point_version: String,
}

pub fn parse_hex_string(hex_str: &str) -> Bytes {
    if hex_str.is_empty() || hex_str == "0x" {
        Bytes::default()
    } else {
        Bytes::from_str(hex_str).unwrap()
    }
}

pub fn parse_u256_from_hex(hex_str: &str) -> U256 {
    U256::from_str(hex_str).unwrap()
}

pub fn parse_address_from_hex(hex_str: &str) -> Address {
    Address::from_str(hex_str).unwrap()
}

pub fn pack_account_gas_limits(
    call_gas_limit: U256,
    verification_gas_limit: U256,
) -> B256 {
    let mut packed = [0u8; 32];
    let verification_bytes = verification_gas_limit.to_be_bytes::<32>();
    let call_bytes = call_gas_limit.to_be_bytes::<32>();
    packed[16..32].copy_from_slice(&verification_bytes[16..32]);
    packed[0..16].copy_from_slice(&call_bytes[16..32]);
    B256::from_slice(&packed)
}

pub fn pack_gas_fees(
    max_fee_per_gas: U256,
    max_priority_fee_per_gas: U256,
) -> B256 {
    let mut packed = [0u8; 32];
    let priority_bytes = max_priority_fee_per_gas.to_be_bytes::<32>();
    let fee_bytes = max_fee_per_gas.to_be_bytes::<32>();
    packed[16..32].copy_from_slice(&priority_bytes[16..32]);
    packed[0..16].copy_from_slice(&fee_bytes[16..32]);
    B256::from_slice(&packed)
}

pub const TEST_PRIVATE_KEY: &str =
    "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";
pub const TEST_USER_OP_HASH: &str =
    "0x2df06416b0a74c9125b57736a863665a767570fa3a5958735ddf2cc325a23a3e";
pub const TEST_EXPECTED_SIGNATURE: &str = "0xe78c2d68677789bdb8e636848e81c7143301c98828c0730f89e1e2ecbb11ddac483d48515fc52d1b9b98ad59c47a41dc160ee8235435c415f44a44c77a9ed7811b";

pub fn load_userop_from_fixture(
    json_path: &str,
) -> (PackedUserOperation, Address, u64) {
    let json_content =
        std::fs::read_to_string(json_path).expect("Failed to read JSON file");
    let test_data: TestData =
        serde_json::from_str(&json_content).expect("Failed to parse JSON");

    let call_gas_limit =
        parse_u256_from_hex(&test_data.user_operation.call_gas_limit);
    let verification_gas_limit =
        parse_u256_from_hex(&test_data.user_operation.verification_gas_limit);
    let max_fee_per_gas =
        parse_u256_from_hex(&test_data.user_operation.max_fee_per_gas);
    let max_priority_fee_per_gas =
        parse_u256_from_hex(&test_data.user_operation.max_priority_fee_per_gas);

    let account_gas_limits =
        pack_account_gas_limits(call_gas_limit, verification_gas_limit);
    let gas_fees = pack_gas_fees(max_fee_per_gas, max_priority_fee_per_gas);

    let user_op = PackedUserOperation {
        sender: parse_address_from_hex(&test_data.user_operation.sender),
        nonce: parse_u256_from_hex(&test_data.user_operation.nonce),
        initCode: parse_hex_string(&test_data.user_operation.init_code),
        callData: parse_hex_string(&test_data.user_operation.call_data),
        accountGasLimits: account_gas_limits,
        preVerificationGas: parse_u256_from_hex(
            &test_data.user_operation.pre_verification_gas,
        ),
        gasFees: gas_fees,
        paymasterAndData: parse_hex_string(
            &test_data.user_operation.paymaster_and_data,
        ),
        signature: parse_hex_string(&test_data.user_operation.signature),
    };

    let entry_point =
        parse_address_from_hex(&test_data.metadata.entry_point_address);
    let chain_id = test_data.metadata.chain_id;

    (user_op, entry_point, chain_id)
}
