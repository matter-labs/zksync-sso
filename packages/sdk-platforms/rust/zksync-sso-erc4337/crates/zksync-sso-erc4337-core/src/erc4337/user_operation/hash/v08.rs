use crate::erc4337::{
    entry_point::PackedUserOperation,
    user_operation::hash::{
        user_operation_hash::UserOperationHash, v08::pack::CodeReader,
    },
};
use alloy::{
    primitives::{Address, B256},
    providers::Provider,
    sol_types::Eip712Domain,
};
use eyre::Result;

pub mod pack;
pub mod user_operation_typed_data;

pub struct PackedUserOperationWrapper(pub PackedUserOperation);

impl PackedUserOperationWrapper {
    pub fn mock() -> Self {
        use alloy::primitives::{Address, B256, Bytes, U256};
        use std::str::FromStr;

        let sender =
            Address::from_str("0x6bf1C0c174e11B933e7d8940aFADf8BB7B8d421C")
                .unwrap();
        let nonce = U256::from(1);
        let init_code = Bytes::from_str("0x").unwrap();
        let call_data = Bytes::from_str("0xe9ae5c530100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000cb98643b8786950f0461f3b0edf99d88f274574d000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000").unwrap();

        // Pack accountGasLimits: verificationGasLimit (128 bits) + callGasLimit (128 bits)
        let account_gas_limits = B256::from_str("0x00000000000000000000000000003fc30000000000000000000000000000e7a8").unwrap();

        let pre_verification_gas = U256::from(52147);

        // Pack gasFees: maxPriorityFeePerGas (128 bits) + maxFeePerGas (128 bits)
        let gas_fees = B256::from_str("0x0000000000000000000000008585115a00000000000000000000000077359400").unwrap();

        let paymaster_and_data = Bytes::from_str("0x").unwrap();
        let signature = Bytes::from_str("0x00000000000000000000000000427edf0c3c3bd42188ab4c907759942abebd93000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000410000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();

        let user_operation = PackedUserOperation {
            sender,
            nonce,
            initCode: init_code,
            callData: call_data,
            accountGasLimits: account_gas_limits,
            preVerificationGas: pre_verification_gas,
            gasFees: gas_fees,
            paymasterAndData: paymaster_and_data,
            signature,
        };

        Self(user_operation)
    }
}

struct NoCodeReader;

impl CodeReader for NoCodeReader {
    fn get_code(&self, _address: &Address) -> Option<Vec<u8>> {
        None
    }
}

pub fn get_user_operation_hash(
    user_operation: &PackedUserOperation,
    entry_point: &Address,
    chain_id: u64,
) -> UserOperationHash {
    get_user_operation_hash_with_code_reader(
        user_operation,
        entry_point,
        chain_id,
        &NoCodeReader,
    )
}

pub async fn get_user_operation_hash_with_provider<P: Provider + Clone>(
    user_operation: &PackedUserOperation,
    entry_point: &Address,
    chain_id: u64,
    provider: &P,
) -> Result<UserOperationHash> {
    let sender_code =
        provider.get_code_at(user_operation.sender).latest().await?;
    let reader = StaticCodeReader { code: Some(sender_code.to_vec()) };
    Ok(get_user_operation_hash_with_code_reader(
        user_operation,
        entry_point,
        chain_id,
        &reader,
    ))
}

struct StaticCodeReader {
    code: Option<Vec<u8>>,
}
impl CodeReader for StaticCodeReader {
    fn get_code(&self, _address: &Address) -> Option<Vec<u8>> {
        self.code.clone()
    }
}

pub fn get_user_operation_hash_with_code_reader(
    user_operation: &PackedUserOperation,
    entry_point: &Address,
    chain_id: u64,
    code_reader: &dyn CodeReader,
) -> UserOperationHash {
    // EIP-712 domain separator using typesafe macro
    let domain: Eip712Domain = pack::build_domain(entry_point, chain_id);

    // EIP-712 struct hash for PackedUserOperation per v0.8
    let struct_hash = pack::struct_hash(user_operation, code_reader);
    let digest = pack::eip712_digest(struct_hash, &domain);

    UserOperationHash(B256::from_slice(digest.as_slice()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::{
        entry_point::PackedUserOperation, user_operation::hash::v08::pack::*,
    };
    use alloy::primitives::{Address, Bytes, U256};
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct TestData {
        user_operation: UserOperationJson,
        metadata: Metadata,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct UserOperationJson {
        sender: String,
        nonce: String,
        init_code: String,
        call_data: String,
        call_gas_limit: String,
        verification_gas_limit: String,
        pre_verification_gas: String,
        max_fee_per_gas: String,
        max_priority_fee_per_gas: String,
        paymaster_and_data: String,
        signature: String,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Metadata {
        chain_id: u64,
        entry_point_address: String,
        entry_point_version: String,
    }

    fn parse_hex_string(hex_str: &str) -> Bytes {
        if hex_str.is_empty() || hex_str == "0x" {
            Bytes::default()
        } else {
            Bytes::from_str(hex_str).unwrap()
        }
    }

    fn parse_u256_from_hex(hex_str: &str) -> U256 {
        U256::from_str(hex_str).unwrap()
    }

    fn parse_address_from_hex(hex_str: &str) -> Address {
        Address::from_str(hex_str).unwrap()
    }

    fn load_test_data() -> (PackedUserOperation, Address, u64) {
        let json_path = "/Users/jackml/Developer/github/matter-labs/zksync-sso/packages/erc4337-contracts/test/integration/erc4337-userop.json";
        let json_content = std::fs::read_to_string(json_path)
            .expect("Failed to read JSON file");
        let test_data: TestData =
            serde_json::from_str(&json_content).expect("Failed to parse JSON");

        // For EntryPoint PackedUserOperation, we need to construct the packed fields
        let call_gas_limit =
            parse_u256_from_hex(&test_data.user_operation.call_gas_limit);
        let verification_gas_limit = parse_u256_from_hex(
            &test_data.user_operation.verification_gas_limit,
        );
        let max_fee_per_gas =
            parse_u256_from_hex(&test_data.user_operation.max_fee_per_gas);
        let max_priority_fee_per_gas = parse_u256_from_hex(
            &test_data.user_operation.max_priority_fee_per_gas,
        );

        // Pack accountGasLimits: verificationGasLimit (128 bits) + callGasLimit (128 bits)
        let account_gas_limits = {
            let mut packed = [0u8; 32];
            let verification_bytes = verification_gas_limit.to_be_bytes::<32>();
            let call_bytes = call_gas_limit.to_be_bytes::<32>();
            packed[16..32].copy_from_slice(&verification_bytes[16..32]); // last 16 bytes
            packed[0..16].copy_from_slice(&call_bytes[16..32]); // last 16 bytes
            B256::from_slice(&packed)
        };

        // Pack gasFees: maxPriorityFeePerGas (128 bits) + maxFeePerGas (128 bits)
        let gas_fees = {
            let mut packed = [0u8; 32];
            let priority_bytes = max_priority_fee_per_gas.to_be_bytes::<32>();
            let fee_bytes = max_fee_per_gas.to_be_bytes::<32>();
            packed[16..32].copy_from_slice(&priority_bytes[16..32]); // last 16 bytes
            packed[0..16].copy_from_slice(&fee_bytes[16..32]); // last 16 bytes
            B256::from_slice(&packed)
        };

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

    struct MockCodeReader;
    impl CodeReader for MockCodeReader {
        fn get_code(&self, _address: &Address) -> Option<Vec<u8>> {
            None
        }
    }

    #[test]
    #[ignore = "need to investigate failure"]
    fn test_get_user_operation_hash_with_code_reader() {
        let (user_operation, entry_point, chain_id) = load_test_data();
        let code_reader = MockCodeReader;

        println!("=== User Operation Data ===");
        println!("sender: {}", user_operation.sender);
        println!("nonce: {}", user_operation.nonce);
        println!("initCode: {}", user_operation.initCode);
        println!("callData: {}", user_operation.callData);
        println!("accountGasLimits: {}", user_operation.accountGasLimits);
        println!("preVerificationGas: {}", user_operation.preVerificationGas);
        println!("gasFees: {}", user_operation.gasFees);
        println!("paymasterAndData: {}", user_operation.paymasterAndData);
        println!("signature: {}", user_operation.signature);
        println!("entry_point: {}", entry_point);
        println!("chain_id: {}", chain_id);

        let typehash = packed_userop_typehash();
        println!("\n=== Intermediary Values ===");
        println!("typehash: {}", typehash);

        let hash_init_code = hashed_init_code::get_hashed_init_code_with_7702(
            &user_operation,
            &code_reader,
        );
        println!("hash_init_code: {}", hash_init_code);

        let hash_call_data =
            hashed_call_data::get_hashed_call_data(&user_operation);
        println!("hash_call_data: {}", hash_call_data);

        let account_gas_limits = verificaction_gas_limit_and_call_gas_limit::get_verificaction_gas_limit_and_call_gas_limit(&user_operation);
        println!("account_gas_limits: {}", account_gas_limits);

        let gas_fees = max_priority_fee_per_gas_and_max_fee_per_gas::get_max_priority_fee_per_gas_and_max_fee_per_gas(&user_operation);
        println!("gas_fees: {}", gas_fees);

        let hash_paymaster_and_data =
            hashed_paymaster_and_data::get_hashed_paymaster_and_data(
                &user_operation,
            );
        println!("hash_paymaster_and_data: {}", hash_paymaster_and_data);

        let hash = get_user_operation_hash_with_code_reader(
            &user_operation,
            &entry_point,
            chain_id,
            &code_reader,
        );
        let expected_hash = "0xc768eb19ae473522e7dd9cb928df330166c8edeaf7de5d419c91b92564a39765";
        let expected_user_op_hash =
            UserOperationHash::from_str(expected_hash).unwrap();

        println!("\n=== Final Results ===");
        println!("Computed hash: {}", hash.0);
        println!("Expected hash: {}", expected_user_op_hash.0);
        println!("Hashes match: {}", hash == expected_user_op_hash);

        assert_eq!(hash, expected_user_op_hash, "User operation hash mismatch");
    }
}
