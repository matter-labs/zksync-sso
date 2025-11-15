use crate::erc4337::{
    entry_point::contract::PackedUserOperation as EntryPointPackedUserOperation,
    user_operation::PackedUserOperation,
};
use alloy::{
    primitives::{Address, Bytes, U256},
    rpc::types::erc4337::PackedUserOperation as AlloyPackedUserOperation,
};

impl From<PackedUserOperation> for AlloyPackedUserOperation {
    fn from(val: PackedUserOperation) -> Self {
        convert_to_alloy_packed_user_op(&val)
    }
}

// Convert from our entry_point::PackedUserOperation to alloy's PackedUserOperation
fn convert_to_alloy_packed_user_op(
    op: &EntryPointPackedUserOperation,
) -> AlloyPackedUserOperation {
    // Extract factory and factory_data from initCode
    let (factory, factory_data) = if op.initCode.len() >= 20 {
        let factory_addr = Some(Address::from_slice(&op.initCode[0..20]));
        let factory_data_bytes = if op.initCode.len() > 20 {
            Some(Bytes::from(op.initCode[20..].to_vec()))
        } else {
            None
        };
        (factory_addr, factory_data_bytes)
    } else {
        (None, None)
    };

    // Extract gas limits from accountGasLimits
    let (verification_gas_limit, call_gas_limit) = if op.accountGasLimits.len()
        == 32
    {
        let mut verification_bytes = [0u8; 32];
        let mut call_bytes = [0u8; 32];
        verification_bytes[16..].copy_from_slice(&op.accountGasLimits[0..16]);
        call_bytes[16..].copy_from_slice(&op.accountGasLimits[16..32]);
        (
            U256::from_be_bytes(verification_bytes),
            U256::from_be_bytes(call_bytes),
        )
    } else {
        (U256::ZERO, U256::ZERO)
    };

    // Extract gas fees from gasFees
    let (max_priority_fee_per_gas, max_fee_per_gas) = if op.gasFees.len() == 32
    {
        let mut priority_bytes = [0u8; 32];
        let mut fee_bytes = [0u8; 32];
        priority_bytes[16..].copy_from_slice(&op.gasFees[0..16]);
        fee_bytes[16..].copy_from_slice(&op.gasFees[16..32]);
        (U256::from_be_bytes(priority_bytes), U256::from_be_bytes(fee_bytes))
    } else {
        (U256::ZERO, U256::ZERO)
    };

    // Extract paymaster data from paymasterAndData
    let (
        paymaster,
        paymaster_verification_gas_limit,
        paymaster_post_op_gas_limit,
        paymaster_data,
    ) = if op.paymasterAndData.len() >= 20 {
        let paymaster_addr =
            Some(Address::from_slice(&op.paymasterAndData[0..20]));

        let (verification_gas, post_op_gas, data) = if op.paymasterAndData.len()
            >= 52
        {
            let mut verification_bytes = [0u8; 32];
            let mut post_op_bytes = [0u8; 32];
            verification_bytes[16..]
                .copy_from_slice(&op.paymasterAndData[20..36]);
            post_op_bytes[16..].copy_from_slice(&op.paymasterAndData[36..52]);

            let data = if op.paymasterAndData.len() > 52 {
                Some(Bytes::from(op.paymasterAndData[52..].to_vec()))
            } else {
                None
            };

            (
                Some(U256::from_be_bytes(verification_bytes)),
                Some(U256::from_be_bytes(post_op_bytes)),
                data,
            )
        } else {
            (None, None, None)
        };

        (paymaster_addr, verification_gas, post_op_gas, data)
    } else {
        (None, None, None, None)
    };

    AlloyPackedUserOperation {
        sender: op.sender,
        nonce: op.nonce,
        factory,
        factory_data,
        call_data: op.callData.clone(),
        call_gas_limit,
        verification_gas_limit,
        pre_verification_gas: op.preVerificationGas,
        max_fee_per_gas,
        max_priority_fee_per_gas,
        paymaster,
        paymaster_verification_gas_limit,
        paymaster_post_op_gas_limit,
        paymaster_data,
        signature: op.signature.clone(),
    }
}
