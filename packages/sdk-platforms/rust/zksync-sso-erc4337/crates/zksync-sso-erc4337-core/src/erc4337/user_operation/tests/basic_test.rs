use crate::erc4337::{
    entry_point::PackedUserOperation,
    user_operation::wrapper_v07::PackedUserOperationWrapperV07,
};
use alloy::{
    node_bindings::Anvil,
    primitives::{Address, Bytes, FixedBytes, U256, address},
};
use alloy_provider::ProviderBuilder;
use std::str::FromStr;

impl PackedUserOperationWrapperV07 {
    pub fn mock() -> Self {
        let sender = address!("a3aBDC7f6334CD3EE466A115f30522377787c024");
        let nonce = U256::from(16);
        let init_code = Bytes::from_str("").unwrap();
        let call_data = Bytes::from_str("b61d27f6000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa9604500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000568656c6c6f000000000000000000000000000000000000000000000000000000").unwrap();

        let max_fee_per_gas = U256::from(17578054897u64);
        let max_priority_fee_per_gas = U256::from(1138018869u64);

        // Pack verification and call gas limits into 32 bytes
        let verification_gas_limit = U256::from(68389);
        let call_gas_limit = U256::from(80000);
        let mut account_gas_limits_bytes = [0u8; 32];
        account_gas_limits_bytes[0..16]
            .copy_from_slice(&verification_gas_limit.to_be_bytes::<16>());
        account_gas_limits_bytes[16..32]
            .copy_from_slice(&call_gas_limit.to_be_bytes::<16>());
        let account_gas_limits =
            FixedBytes::<32>::from(account_gas_limits_bytes);

        // Pack priority and max fee into 32 bytes
        let mut gas_fees_bytes = [0u8; 32];
        gas_fees_bytes[0..16]
            .copy_from_slice(&max_priority_fee_per_gas.to_be_bytes::<16>());
        gas_fees_bytes[16..32]
            .copy_from_slice(&max_fee_per_gas.to_be_bytes::<16>());
        let gas_fees = FixedBytes::<32>::from(gas_fees_bytes);

        let signature = Bytes::from_str("a15569dd8f8324dbeabf8073fdec36d4b754f53ce5901e283c6de79af177dc94557fa3c9922cd7af2a96ca94402d35c39f266925ee6407aeb32b31d76978d4ba1c").unwrap();
        let pre_verification_gas = U256::from(55721);

        // Combine paymaster data
        let paymaster_and_data = Bytes::from_str("0000000000000039cd5e8aE05257CE51C473ddd100000066cc6b8b000000000000bce787423a07dde9c43cdf50ff33bf35b18babd336cc9739fd9f6dca86e200934505c311454b60c3aa1d206e6bb893f3489e77ace4c58f30d47cebd368a1422a1c").unwrap();

        Self(PackedUserOperation {
            sender,
            nonce,
            initCode: init_code,
            callData: call_data,
            accountGasLimits: account_gas_limits,
            preVerificationGas: pre_verification_gas,
            gasFees: gas_fees,
            paymasterAndData: paymaster_and_data,
            signature,
        })
    }
}

#[tokio::test]
async fn test_send_user_operation() -> eyre::Result<()> {
    let anvil = Anvil::new();
    let anvil_instance = anvil.spawn();
    let anvil_url = anvil_instance.endpoint_url();
    let wallet = anvil_instance.wallet().unwrap();

    let _provider =
        ProviderBuilder::new().wallet(wallet).connect_http(anvil_url);

    // TODO
    // 1. deploy contracts
    // 2. run alto

    let _receipient = Address::default();

    // let execution_lib_instance = {
    //     use crate::erc4337::user_operation::v08::contracts::ExecutionLib;
    //     let execution_lib_address = Address::default(); // TODO
    //     ExecutionLib::new(execution_lib_address, provider)
    // };
    // let execution = execution_lib_instance.encode_single

    // use crate::erc4337::account::erc7579::account::IERC7579Account;

    // function test_transfer() public {
    //     address recipient = makeAddr("recipient");
    //     bytes memory execution = ExecutionLib.encodeSingle(recipient, 1 ether, "");
    //     bytes memory callData = abi.encodeCall(IERC7579Account.execute, (ModeLib.encodeSimpleSingle(), execution));
    //     PackedUserOperation[] memory userOps = new PackedUserOperation[](1);
    //     userOps[0] = makeSignedUserOp(callData, owner.key, address(eoaValidator));

    //     entryPoint.handleOps(userOps, bundler);
    //     vm.assertEq(recipient.balance, 1 ether, "Value not transferred via simple call");
    // }

    Ok(())
}
