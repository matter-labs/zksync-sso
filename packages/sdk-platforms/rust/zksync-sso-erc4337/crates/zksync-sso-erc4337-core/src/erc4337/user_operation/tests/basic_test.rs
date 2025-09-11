use crate::erc4337::user_operation::wrapper::PackedUserOperationWrapper;
use alloy::{
    node_bindings::{Anvil, AnvilInstance},
    primitives::{Address, Bytes, U256, address},
    rpc::types::erc4337::PackedUserOperation,
};
use alloy_provider::ProviderBuilder;
use std::str::FromStr;

impl PackedUserOperationWrapper {
    pub fn mock() -> Self {
        let sender = address!("a3aBDC7f6334CD3EE466A115f30522377787c024");
        let nonce = U256::from(16);
        let factory: Option<Address> = None;
        let factory_data: Option<Bytes> = None;
        let call_data = Bytes::from_str("b61d27f6000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa9604500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000568656c6c6f000000000000000000000000000000000000000000000000000000").unwrap();

        let max_fee_per_gas = U256::from(17578054897u64);
        let max_priority_fee_per_gas = U256::from(1138018869u64);

        let signature = Bytes::from_str("a15569dd8f8324dbeabf8073fdec36d4b754f53ce5901e283c6de79af177dc94557fa3c9922cd7af2a96ca94402d35c39f266925ee6407aeb32b31d76978d4ba1c").unwrap();
        let call_gas_limit = U256::from(80000);
        let verification_gas_limit = U256::from(68389);
        let pre_verification_gas = U256::from(55721);
        let paymaster = Some(
            "0x0000000000000039cd5e8aE05257CE51C473ddd1"
                .parse::<Address>()
                .unwrap(),
        );
        let paymaster_verification_gas_limit = Some(U256::from(27776));
        let paymaster_post_op_gas_limit = Some(U256::from(1));
        let paymaster_data = Some(Bytes::from_str("00000066cc6b8b000000000000bce787423a07dde9c43cdf50ff33bf35b18babd336cc9739fd9f6dca86e200934505c311454b60c3aa1d206e6bb893f3489e77ace4c58f30d47cebd368a1422a1c").unwrap());

        Self(PackedUserOperation {
            sender,
            nonce,
            factory,
            factory_data,
            call_data,
            call_gas_limit,
            verification_gas_limit,
            pre_verification_gas,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            paymaster,
            paymaster_verification_gas_limit,
            paymaster_post_op_gas_limit,
            paymaster_data,
            signature,
        })
    }
}

// function test_transfer() public {
//     address recipient = makeAddr("recipient");
//     bytes memory execution = ExecutionLib.encodeSingle(recipient, 1 ether, "");
//     bytes memory callData = abi.encodeCall(IERC7579Account.execute, (ModeLib.encodeSimpleSingle(), execution));
//     PackedUserOperation[] memory userOps = new PackedUserOperation[](1);
//     userOps[0] = makeSignedUserOp(callData, owner.key, address(eoaValidator));

//     entryPoint.handleOps(userOps, bundler);
//     vm.assertEq(recipient.balance, 1 ether, "Value not transferred via simple call");
// }
//

#[tokio::test]
async fn test_send_user_operation() -> eyre::Result<()> {
    let anvil = Anvil::new();
    let anvil_instance = anvil.spawn();
    let anvil_url = anvil_instance.endpoint_url();
    let wallet = anvil_instance.wallet().unwrap();

    let provider =
        ProviderBuilder::new().wallet(wallet).connect_http(anvil_url);

    // TODO
    // 1. deploy contracts
    // 2. run alto

    let receipient = Address::default();

    // let execution_lib_instance = {
    //     use crate::erc4337::user_operation::v08::contracts::ExecutionLib;
    //     let execution_lib_address = Address::default(); // TODO
    //     ExecutionLib::new(execution_lib_address, provider)
    // };
    // let execution = execution_lib_instance.encode_single

    use crate::erc4337::account::erc7579::account::IERC7579Account;

    //     bytes memory execution = ExecutionLib.encodeSingle(recipient, 1 ether, "");
    //     bytes memory callData = abi.encodeCall(IERC7579Account.execute, (ModeLib.encodeSimpleSingle(), execution));
    //     PackedUserOperation[] memory userOps = new PackedUserOperation[](1);
    //     userOps[0] = makeSignedUserOp(callData, owner.key, address(eoaValidator));

    //     entryPoint.handleOps(userOps, bundler);
    //     vm.assertEq(recipient.balance, 1 ether, "Value not transferred via simple call");

    Ok(())
}
