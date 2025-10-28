use crate::erc4337::{
    entry_point::{EntryPoint, PackedUserOperation},
    user_operation::hash::user_operation_hash::UserOperationHash,
};
use alloy::{primitives::Address, providers::Provider};

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

pub async fn get_user_operation_hash_entry_point<P: Provider + Clone>(
    user_operation: &PackedUserOperation,
    entry_point: &Address,
    provider: P,
) -> eyre::Result<UserOperationHash> {
    let entry_point = EntryPoint::new(*entry_point, provider.clone());
    let hash = entry_point.getUserOpHash(user_operation.clone()).call().await?;
    Ok(hash.into())
}
