use alloy::primitives::{Address, Bytes, FixedBytes, U256, keccak256};
use alloy::rpc::types::eth::UserOperation;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::{Signature, Signer};
use alloy::sol;
use alloy::sol_types::SolCall;

#[derive(Debug, Clone)]
pub struct UserOperationSigner {
    signer: PrivateKeySigner,
    entry_point: Address,
    chain_id: u64,
}

impl UserOperationSigner {
    pub fn new(private_key: &str, entry_point: Address, chain_id: u64) -> eyre::Result<Self> {
        let key_bytes = alloy::primitives::hex::decode(private_key.trim_start_matches("0x"))?;
        let signer =
            PrivateKeySigner::from_bytes(&FixedBytes::<32>::try_from(key_bytes.as_slice())?)?;

        Ok(Self {
            signer,
            entry_point,
            chain_id,
        })
    }

    pub fn address(&self) -> Address {
        self.signer.address()
    }

    pub async fn sign_user_operation(
        &self,
        mut user_op: UserOperation,
    ) -> eyre::Result<UserOperation> {
        let user_op_hash = self.calculate_user_op_hash(&user_op)?;
        let signature = self.signer.sign_hash(&user_op_hash).await?;

        user_op.signature = Bytes::from(signature.as_bytes().to_vec());

        Ok(user_op)
    }

    pub fn calculate_user_op_hash(&self, user_op: &UserOperation) -> eyre::Result<FixedBytes<32>> {
        // Pack the UserOperation data (EIP-4337 spec)
        let mut packed = Vec::new();

        // Add all fields except signature
        packed.extend_from_slice(user_op.sender.as_slice());
        packed.extend_from_slice(&user_op.nonce.to_be_bytes::<32>());

        // Hash init_code
        let init_code_hash = keccak256(&user_op.init_code);
        packed.extend_from_slice(init_code_hash.as_slice());

        // Hash call_data
        let call_data_hash = keccak256(&user_op.call_data);
        packed.extend_from_slice(call_data_hash.as_slice());

        packed.extend_from_slice(&user_op.call_gas_limit.to_be_bytes::<32>());
        packed.extend_from_slice(&user_op.verification_gas_limit.to_be_bytes::<32>());
        packed.extend_from_slice(&user_op.pre_verification_gas.to_be_bytes::<32>());
        packed.extend_from_slice(&user_op.max_fee_per_gas.to_be_bytes::<32>());
        packed.extend_from_slice(&user_op.max_priority_fee_per_gas.to_be_bytes::<32>());

        // Hash paymaster_and_data
        let paymaster_data_hash = keccak256(&user_op.paymaster_and_data);
        packed.extend_from_slice(paymaster_data_hash.as_slice());

        // First hash of packed data
        let hash1 = keccak256(&packed);

        // Final hash with entry point and chain ID (EIP-4337 spec)
        let mut final_data = Vec::new();
        final_data.extend_from_slice(hash1.as_slice());
        final_data.extend_from_slice(self.entry_point.as_slice());
        final_data.extend_from_slice(&self.chain_id.to_be_bytes());

        Ok(keccak256(&final_data))
    }

    pub fn verify_user_operation_signature(&self, user_op: &UserOperation) -> eyre::Result<bool> {
        if user_op.signature.is_empty() {
            return Ok(false);
        }

        let user_op_hash = self.calculate_user_op_hash(user_op)?;

        // Create a copy without signature for verification
        let mut unsigned_op = user_op.clone();
        unsigned_op.signature = Bytes::new();

        let signature = Signature::try_from(user_op.signature.as_ref())?;
        let recovered_address = signature.recover_address_from_prehash(&user_op_hash)?;

        Ok(recovered_address == self.signer.address())
    }
}

// Utility functions for UserOperation manipulation
pub mod utils {
    use super::*;

    pub fn create_simple_account_user_operation(
        sender: Address,
        nonce: U256,
        call_data: Bytes,
        call_gas_limit: Option<U256>,
        verification_gas_limit: Option<U256>,
        pre_verification_gas: Option<U256>,
        max_fee_per_gas: Option<U256>,
        max_priority_fee_per_gas: Option<U256>,
    ) -> UserOperation {
        UserOperation {
            sender,
            nonce,
            init_code: Bytes::new(), // Empty for existing accounts
            call_data,
            call_gas_limit: call_gas_limit.unwrap_or_else(|| U256::from(100000u64)),
            verification_gas_limit: verification_gas_limit.unwrap_or_else(|| U256::from(100000u64)),
            pre_verification_gas: pre_verification_gas.unwrap_or_else(|| U256::from(21000u64)),
            max_fee_per_gas: max_fee_per_gas.unwrap_or_else(|| U256::from(20_000_000_000u64)),
            max_priority_fee_per_gas: max_priority_fee_per_gas
                .unwrap_or_else(|| U256::from(1_000_000_000u64)),
            paymaster_and_data: Bytes::new(),
            signature: Bytes::new(), // Will be filled by signer
        }
    }

    pub fn encode_execute_call(target: Address, value: U256, data: Bytes) -> Bytes {
        sol! {
            function execute(address target, uint256 value, bytes calldata data);
        }

        let call = executeCall {
            target,
            value,
            data,
        };

        Bytes::from(call.abi_encode())
    }
}
