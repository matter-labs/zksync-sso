use crate::erc4337::{
    account::{
        erc7579::{account::Execution, calls::encode_calls},
        modular_smart_account::signature::{eoa_signature, stub_signature},
    },
    bundler::pimlico::client::BundlerClient,
    entry_point::EntryPoint,
    user_operation::hash::v08::get_user_operation_hash_entry_point,
};
use alloy::{
    primitives::{Address, Bytes, U256},
    providers::Provider,
    rpc::types::erc4337::{
        PackedUserOperation as AlloyPackedUserOperation, SendUserOperation,
    },
};

pub async fn send_transaction<P: Provider + Send + Sync + Clone>(
    account: Address,
    eoa_validator: Address,
    entry_point: Address,
    calls: Vec<Execution>,
    bundler_client: BundlerClient,
    provider: P,
    private_key_hex: String,
) -> eyre::Result<()> {
    let encoded_calls: Bytes = encode_calls(calls).into();
    // let user_op = {
    //     let stub_sig = stub_signature(eoa_validator);
    //     UserOperationV08 {
    //         sender: account,
    //         call_data: encoded_calls.clone().into(),
    //         signature: Bytes::default(),
    //         ..Default::default()
    //     }
    // };

    let (estimated_gas, mut user_op) = {
        let alloy_user_op = {
            let stub_sig = stub_signature(eoa_validator)?;
            AlloyPackedUserOperation {
                sender: account,
                nonce: Default::default(),
                paymaster: None,
                paymaster_verification_gas_limit: None,
                paymaster_data: None,
                paymaster_post_op_gas_limit: None,
                call_gas_limit: Default::default(),
                max_priority_fee_per_gas: Default::default(),
                max_fee_per_gas: Default::default(),
                pre_verification_gas: Default::default(),
                verification_gas_limit: Default::default(),
                factory: None,
                factory_data: None,
                call_data: encoded_calls,
                signature: stub_sig,
            }
        };
        let send_user_op =
            SendUserOperation::EntryPointV07(alloy_user_op.clone());
        // let bundler_provider = {
        //     let rpc_url = "http://localhost:4337".parse().unwrap();

        //     let signer_private_key = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
        //     let signer = PrivateKeySigner::from_str(&signer_private_key)?;
        //     let alloy_signer = signer.clone();
        //     let ethereum_wallet =
        //         alloy::network::EthereumWallet::new(alloy_signer.clone());

        //     let provider = ProviderBuilder::new()
        //         .wallet(ethereum_wallet.clone())
        //         .connect_http(rpc_url);

        //     provider
        // };
        // bundler_provider
        //     .estimate_user_operation_gas(send_user_op, entry_point)
        //     .await?
        // Error: deserialization error: missing field `verificationGas` at line 1 column 158
        // {
        //     "preVerificationGas":"0xbf1a",
        //     "verificationGasLimit":"0x1c6d1",
        //     "callGasLimit":"0x3fc3",
        //     "paymasterVerificationGasLimit":"0x0",
        //     "paymasterPostOpGasLimit":"0x0"
        // }
        // Caused by:
        //     missing field `verificationGas` at line 1 column 158

        let estimated_gas = bundler_client
            .estimate_user_operation_gas(&alloy_user_op, &entry_point)
            .await?;

        (estimated_gas, alloy_user_op)
    };

    user_op.call_gas_limit = estimated_gas.callGasLimit;
    user_op.verification_gas_limit = estimated_gas.verificationGasLimit;
    user_op.pre_verification_gas = estimated_gas.preVerificationGas;

    let packed_gas_limits: U256 =
        ((user_op.verification_gas_limit << 128) | user_op.call_gas_limit);

    let gas_fees: U256 =
        ((user_op.max_priority_fee_per_gas << 128) | user_op.max_fee_per_gas);

    let packed_user_op = EntryPoint::PackedUserOperation {
        sender: user_op.sender,
        nonce: user_op.nonce,
        initCode: Bytes::default(),
        callData: user_op.call_data.clone(),
        accountGasLimits: packed_gas_limits.to_le_bytes().into(),
        preVerificationGas: user_op.pre_verification_gas,
        gasFees: gas_fees.to_le_bytes().into(),
        paymasterAndData: Bytes::default(),
        signature: user_op.signature.clone(),
    };

    let hash = get_user_operation_hash_entry_point(
        &packed_user_op,
        &entry_point,
        provider,
    )
    .await?;

    dbg!(hash);

    let signature = eoa_signature(private_key_hex, eoa_validator, hash.0)?;
    user_op.signature = signature;

    let user_op_hash =
        bundler_client.send_user_operation(entry_point, user_op).await?;
    
    // params: [
    //     Object {
    //         "callData": String("0xe9ae5c530100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000"),
    //         "callGasLimit": String("0x3fc3"),
    //         "maxFeePerGas": String("0x0"),
    //         "maxPriorityFeePerGas": String("0x0"),
    //         "nonce": String("0x0"),
    //         "preVerificationGas": String("0xbf1a"),
    //         "sender": String("0x09b5508134a3a2e2a99e87f6cd433b6a3a1a7303"),
    //         "signature": String("0x00427edf0c3c3bd42188ab4c907759942abebd9356b1b747b5973eb03e352cf88b70eb5be6cbfc78960e3ac674fb53a709a0baaa19b21dccbe806564e90901bea340116ad521eaed9e4fc660a74bb22a716e106f1c"),
    //         "verificationGasLimit": String("0x1c6d1"),
    //     },
    //     String("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108"),
    // ]

    // response_text: "{\"jsonrpc\":\"2.0\",\"id\":1,\"error\":{\"message\":\"UserOperation reverted with reason: AA24 signature error\",\"code\":-32500}}"
    // raw_payload: JSONRPCResponse { jsonrpc: "2.0", id: 1, result: None, error: Some(ErrorPayload { message: "UserOperation reverted with reason: AA24 signature error", data: None, code: Some(-32500) }) }
    // Error: ErrorPayload { message: UserOperation reverted with reason: AA24 signature error }
    
    dbg!(user_op_hash);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::account::modular_smart_account::deploy::{
        EOASigners, deploy_account_basic, is_module_installed,
    };
    use alloy::{
        primitives::{Bytes, U256, address},
        providers::ProviderBuilder,
        rpc::types::TransactionRequest,
        signers::local::PrivateKeySigner,
    };
    use std::str::FromStr;

    #[tokio::test]
    async fn test_send_transaction() -> eyre::Result<()> {
        let rpc_url = "http://localhost:8545".parse()?;

        let factory_address =
            address!("0x679FFF51F11C3f6CaC9F2243f9D14Cb1255F65A3");

        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        let eoa_validator_address =
            address!("0x00427eDF0c3c3bd42188ab4C907759942Abebd93");

        let signer_private_key = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

        let provider = {
            let signer = PrivateKeySigner::from_str(&signer_private_key)?;
            let alloy_signer = signer.clone();
            let ethereum_wallet =
                alloy::network::EthereumWallet::new(alloy_signer.clone());

            let provider = ProviderBuilder::new()
                .wallet(ethereum_wallet.clone())
                .connect_http(rpc_url);

            provider
        };

        let signers =
            vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let address = deploy_account_basic(
            factory_address,
            Some(eoa_signers),
            provider.clone(),
        )
        .await?;

        println!("Account deployed");

        let is_module_installed = is_module_installed(
            eoa_validator_address,
            address,
            provider.clone(),
        )
        .await?;
        eyre::ensure!(is_module_installed, "Module is not installed");

        let call = {
            let target = address!("0x0000000000000000000000000000000000000000");
            let value = U256::from(1);
            let data = Bytes::default();
            Execution { target, value, data }
        };

        let calls = vec![call];

        let bundler_client = {
            use crate::erc4337::bundler::config::BundlerConfig;
            let bundler_url = "http://localhost:4337".to_string();
            let config = BundlerConfig::new(bundler_url);
            BundlerClient::new(config)
        };

        {
            let fund_tx = TransactionRequest::default()
                .to(address)
                .value(U256::from(10000000000000000000u64));
            _ = provider.send_transaction(fund_tx).await?.get_receipt().await?;
        }

        let response = send_transaction(
            address,
            eoa_validator_address,
            entry_point_address,
            calls,
            bundler_client,
            provider.clone(),
            signer_private_key.to_string(),
        )
        .await?;

        dbg!(response);

        Ok(())
    }
}
