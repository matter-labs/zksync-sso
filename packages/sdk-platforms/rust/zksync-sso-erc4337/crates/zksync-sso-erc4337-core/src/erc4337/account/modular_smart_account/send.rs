use crate::erc4337::account::modular_smart_account::nonce::get_nonce;
use crate::erc4337::{
    account::{
        erc7579::{account::Execution, calls::encode_calls},
        modular_smart_account::signature::{eoa_signature, stub_signature},
    },
    bundler::pimlico::{client::BundlerClient, estimate::Estimate},
    entry_point::EntryPoint,
    user_operation::hash::v08::get_user_operation_hash_entry_point,
};
use alloy::{
    primitives::{
        Address, Bytes, FixedBytes, U256, address, bytes, fixed_bytes,
    },
    providers::Provider,
    rpc::types::erc4337::{
        PackedUserOperation as AlloyPackedUserOperation, SendUserOperation,
    },
};
use alloy::{
    providers::ProviderBuilder, rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use alloy_provider::ext::Erc4337Api;
use std::str::FromStr;

pub async fn send_transaction<P: Provider + Send + Sync + Clone>(
    account: Address,
    eoa_validator: Address,
    entry_point: Address,
    call_data: Bytes,
    bundler_client: BundlerClient,
    provider: P,
    private_key_hex: String,
) -> eyre::Result<()> {
    // {
    //     let expected_account: Address =
    //         address!("0x6bf1c0c174e11b933e7d8940afadf8bb7b8d421c");
    //     eyre::ensure!(
    //         account == expected_account,
    //         "account should be: {expected_account}, received: {account}"
    //     );
    // };

    // let expected_encoded_calls_hex = "0xe9ae5c530100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000006bf1c0c174e11b933e7d8940afadf8bb7b8d421c000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000";
    // let expected_encoded_calls = Bytes::from_str(expected_encoded_calls_hex)?;
    // eyre::ensure!(
    //     encoded_calls == expected_encoded_calls,
    //     "Encoded calls do not match expected, received: {:?}, expected: {:?}",
    //     encoded_calls,
    //     expected_encoded_calls,
    // );

    let (estimated_gas, mut user_op) = {
        let alloy_user_op = {
            let stub_sig = stub_signature(eoa_validator)?;
            AlloyPackedUserOperation {
                sender: account,
                nonce: get_nonce(entry_point, account, &provider).await?,
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
                call_data: call_data,
                signature: stub_sig,
            }
        };
        // let send_user_op =
        //     SendUserOperation::EntryPointV07(alloy_user_op.clone());
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

        // let estimated_gas = Estimate {
        //     preVerificationGas: U256::from(49186),
        //     verificationGasLimit: U256::from(116433),
        //     callGasLimit: U256::from(16323),
        //     paymasterVerificationGasLimit: U256::from(0),
        //     paymasterPostOpGasLimit: U256::from(0),
        // };

        (estimated_gas, alloy_user_op)
    };

    user_op.call_gas_limit = estimated_gas.callGasLimit;
    user_op.verification_gas_limit = estimated_gas.verificationGasLimit;
    user_op.pre_verification_gas = estimated_gas.preVerificationGas;

    user_op.max_priority_fee_per_gas = U256::from(0x77359400);
    user_op.max_fee_per_gas = U256::from(0x82e08afeu64);

    let packed_gas_limits: U256 =
        ((user_op.verification_gas_limit << 128) | user_op.call_gas_limit);

    let gas_fees: U256 =
        ((user_op.max_priority_fee_per_gas << 128) | user_op.max_fee_per_gas);

    let packed_user_op = EntryPoint::PackedUserOperation {
        sender: user_op.sender,
        nonce: user_op.nonce,
        initCode: Bytes::default(),
        callData: user_op.call_data.clone(),
        accountGasLimits: packed_gas_limits.to_be_bytes().into(),
        preVerificationGas: user_op.pre_verification_gas,
        gasFees: gas_fees.to_be_bytes().into(),
        paymasterAndData: Bytes::default(),
        signature: user_op.signature.clone(),
    };

    // {
    //     let expected_call_data = Bytes::from_str(
    //         "0xe9ae5c530100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000006bf1c0c174e11b933e7d8940afadf8bb7b8d421c000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000",
    //     )?;
    //     let call_data = user_op.clone().call_data;
    //     eyre::ensure!(
    //         call_data == expected_call_data,
    //         "call_data should be: {expected_call_data}, received: {call_data}"
    //     );
    // }

    // {
    //     let expected_call_gas_limits: U256 = U256::from(0x3fc3);
    //     let call_gas_limits: U256 = user_op.clone().call_gas_limit;
    //     eyre::ensure!(
    //         call_gas_limits == expected_call_gas_limits,
    //         "call_gas_limits should be: {expected_call_gas_limits}, received: {call_gas_limits}"
    //     );
    // };

    // {
    //     let expected_verification_gas_limits: U256 = U256::from(0x1c6d1);
    //     let verification_gas_limits: U256 =
    //         user_op.clone().verification_gas_limit;
    //     eyre::ensure!(
    //         expected_verification_gas_limits == verification_gas_limits,
    //         "verification gas limit should be: {expected_verification_gas_limits}, received: {verification_gas_limits}"
    //     );
    // };

    // {
    //     let expected_max_priority_fee_per_gas: U256 = U256::from(0x77359400);
    //     let max_priority_fee_per_gas: U256 =
    //         user_op.clone().max_priority_fee_per_gas;
    //     eyre::ensure!(
    //         max_priority_fee_per_gas == expected_max_priority_fee_per_gas,
    //         "max_priority_fee_per_gas should be: {expected_max_priority_fee_per_gas}, received: {max_priority_fee_per_gas}"
    //     );
    // };

    // {
    //     let expected_max_fee_per_gas: U256 = U256::from(0x82e08afeu64);
    //     let max_fee_per_gas: U256 = user_op.clone().max_fee_per_gas;
    //     eyre::ensure!(
    //         max_fee_per_gas == expected_max_fee_per_gas,
    //         "max_fee_per_gas should be: {expected_max_fee_per_gas}, received: {max_fee_per_gas}"
    //     );
    // };

    // {
    //     let expected_nonce: U256 = U256::from(0x0);
    //     let nonce: U256 = user_op.clone().nonce;
    //     eyre::ensure!(
    //         nonce == expected_nonce,
    //         "nonce should be: {expected_nonce}, received: {nonce}"
    //     );
    // };

    // {
    //     let expected_pre_verification_gas: U256 = U256::from(0xc022);
    //     let pre_verification_gas: U256 = user_op.clone().pre_verification_gas;
    //     eyre::ensure!(
    //         pre_verification_gas == expected_pre_verification_gas,
    //         "pre_verification_gas should be: {expected_pre_verification_gas}, received: {pre_verification_gas}"
    //     );
    // };

    // {
    //     let expected_sender: Address =
    //         address!("0x6bf1c0c174e11b933e7d8940afadf8bb7b8d421c");
    //     let sender: Address = user_op.clone().sender;
    //     eyre::ensure!(
    //         sender == expected_sender,
    //         "sender should be: {expected_sender}, received: {sender}"
    //     );
    // };

    let hash = get_user_operation_hash_entry_point(
        &packed_user_op,
        &entry_point,
        provider,
    )
    .await?;

    dbg!(hash);

    // {
    //     let expected_hash: FixedBytes<32> = "0x9ceec43dc797fa8dee197ba919967bdc8ee34bda38d2464a2fde666ba85b509d".parse().unwrap();
    //     eyre::ensure!(
    //         hash.0 == expected_hash,
    //         "hash should be: {expected_hash}, received: {}",
    //         hash.0
    //     );
    //     // Error: hash should be: 0x9ceec43dc797fa8dee197ba919967bdc8ee34bda38d2464a2fde666ba85b509d,
    //     //              received: 0xb64ba314b77db473c08bb44171ae71f2bd2fcbc51a302d8feba0a6e47c9adfa1
    // }
    let signature = eoa_signature(private_key_hex, eoa_validator, hash.0)?;
    user_op.signature = signature;

    // {
    //     let expected_signature: Bytes = bytes!(
    //         "0x00427edf0c3c3bd42188ab4c907759942abebd939be194ec9c86d5d6e2ae115ce700496a0a02f324f281cb9e0ead1e1346dccf241bf51064257e4e6e99208a3d80ef752c2891b586d4862aaf2ef5fe2f94aecaef1b"
    //     );
    //     let signature: Bytes = user_op.clone().signature;
    //     eyre::ensure!(
    //         signature == expected_signature,
    //         "signature should be: {expected_signature}, received: {signature}"
    //     );
    // };

    {
        let expected_entry_point: Address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");
        eyre::ensure!(
            entry_point == expected_entry_point,
            "entry_point should be: {expected_entry_point}, received: {entry_point}"
        );
    };

    // sendUserOperation rpcParameters:  {
    //   callData: '0xe9ae5c530100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000006bf1c0c174e11b933e7d8940afadf8bb7b8d421c000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000',
    //   callGasLimit: '0x3fc3',
    //   maxFeePerGas: '0x82e08afe',
    //   maxPriorityFeePerGas: '0x77359400',
    //   nonce: '0x0',
    //   paymasterPostOpGasLimit: '0x0',
    //   paymasterVerificationGasLimit: '0x0',
    //   preVerificationGas: '0xc022',
    //   sender: '0x6bf1c0c174e11b933e7d8940afadf8bb7b8d421c',
    //   signature: '0x00427edf0c3c3bd42188ab4c907759942abebd939be194ec9c86d5d6e2ae115ce700496a0a02f324f281cb9e0ead1e1346dccf241bf51064257e4e6e99208a3d80ef752c2891b586d4862aaf2ef5fe2f94aecaef1b',
    //   verificationGasLimit: '0x1c6d1'
    // }
    // hash: 0x9ceec43dc797fa8dee197ba919967bdc8ee34bda38d2464a2fde666ba85b509d

    let user_op_hash =
        bundler_client.send_user_operation(entry_point, user_op).await?;

    dbg!(user_op_hash.clone());

    // get_user_operation_receipt
    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

    _ = bundler_client.get_user_operation_receipt(user_op_hash).await?;

    // response_text: "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"userOpHash\":\"0x8da0baf4f6168c4c275b257424e08a68b69f7cf1bf5e70e8f671be5804c68625\",\"entryPoint\":\"0x4337084d9e255ff0702461cf8895ce9e3b5ff108\",\"sender\":\"0xDb95C59A5304D296E3F9A647E508046f8A8B5a37\",\"nonce\":\"0x0\",\"actualGasUsed\":\"0x243ec\",\"actualGasCost\":\"0x10e26ec717d08\",\"success\":true,\"logs\":[],\"receipt\":{\"transactionHash\":\"0xb8c390ae0f08bd1264e70f35b3cc09b6fc224ab05fb65e384528388337bd8ad7\",\"transactionIndex\":\"0x0\",\"blockHash\":\"0x40338d3d31372242b051f3cd6aac656f8e7c4a32b8de5dc69588c80ffce18fd0\",\"blockNumber\":\"0x166887c\",\"from\":\"0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266\",\"to\":\"0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108\",\"cumulativeGasUsed\":\"0x25e97\",\"gasUsed\":\"0x25e97\",\"contractAddress\":null,\"logs\":[{\"logIndex\":\"0x0\",\"transactionIndex\":\"0x0\",\"transactionHash\":\"0xb8c390ae0f08bd1264e70f35b3cc09b6fc224ab05fb65e384528388337bd8ad7\",\"blockHash\":\"0x40338d3d31372242b051f3cd6aac656f8e7c4a32b8de5dc69588c80ffce18fd0\",\"blockNumber\":\"0x166887c\",\"address\":\"0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108\",\"data\":\"0x00000000000000000000000000000000000000000000000000016b57cd1f4494\",\"topics\":[\"0x2da466a7b24304f47e87fa2e1e5a81b9831ce54fec19055ce277ca2f39ba42c4\",\"0x000000000000000000000000db95c59a5304d296e3f9a647e508046f8a8b5a37\"]},{\"logIndex\":\"0x1\",\"transactionIndex\":\"0x0\",\"transactionHash\":\"0xb8c390ae0f08bd1264e70f35b3cc09b6fc224ab05fb65e384528388337bd8ad7\",\"blockHash\":\"0x40338d3d31372242b051f3cd6aac656f8e7c4a32b8de5dc69588c80ffce18fd0\",\"blockNumber\":\"0x166887c\",\"address\":\"0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108\",\"data\":\"0x\",\"topics\":[\"0xbb47ee3e183a558b1a2ff0874b079f3fc5478b7454eacf2bfc5af2ff5878f972\"]},{\"logIndex\":\"0x2\",\"transactionIndex\":\"0x0\",\"transactionHash\":\"0xb8c390ae0f08bd1264e70f35b3cc09b6fc224ab05fb65e384528388337bd8ad7\",\"blockHash\":\"0x40338d3d31372242b051f3cd6aac656f8e7c4a32b8de5dc69588c80ffce18fd0\",\"blockNumber\":\"0x166887c\",\"address\":\"0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108\",\"data\":\"0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000010e26ec717d0800000000000000000000000000000000000000000000000000000000000243ec\",\"topics\":[\"0x49628fd1471006c1482da88028e9ce4dbb080b815c9b0344d39e5a8e6ec1419f\",\"0x8da0baf4f6168c4c275b257424e08a68b69f7cf1bf5e70e8f671be5804c68625\",\"0x000000000000000000000000db95c59a5304d296e3f9a647e508046f8a8b5a37\",\"0x0000000000000000000000000000000000000000000000000000000000000000\"]}],\"logsBloom\":\"0x00000000020000000000000000000000000000000000000008000000000000000008000000000000000000010000000000000000000000000000020000000000000000000000000000000000000002000000000000000000000000000000200000000000020000000000000000000800000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000080000000000000000000000002000000000000020000000010000001000000000000000000000000200000000010000020000040000000000000000000000000000000000000000000000000800000000000\",\"status\":\"0x1\",\"effectiveGasPrice\":\"0x3ba6a426\"}}}"

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
    async fn test_send_transaction_contracts() -> eyre::Result<()> {
        let rpc_url = "http://localhost:8545".parse()?;

        // == Logs ==
        //   EOAKeyValidator: 0x00427eDF0c3c3bd42188ab4C907759942Abebd93
        //   SessionKeyValidator: 0x57eaa1Fd8d80135Db195B147a249aad777aD10f0
        //   WebAuthnValidator: 0xF3F924c9bADF6891D3676cfe9bF72e2C78527E17
        //   GuardianExecutor: 0x374ce0d25B00B909417d695237d06abFe4548eB1
        //   ModularSmartAccount implementation: 0x5646c10bFa3fA97B72402D26Bc66fEc0dbAf99c8
        //   UpgradeableBeacon: 0x7b1255B5DaBbBf84ADC423B8b6Ecd89F822A2f72
        //   MSAFactory: 0x679FFF51F11C3f6CaC9F2243f9D14Cb1255F65A3
        //   Initialized account: 0x6bf1C0c174e11B933e7d8940aFADf8BB7B8d421C

        let factory_address =
            address!("0x679FFF51F11C3f6CaC9F2243f9D14Cb1255F65A3");

        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        let eoa_validator_address =
            address!("0x00427eDF0c3c3bd42188ab4C907759942Abebd93");

        let signer_private_key = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";

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
        // let address = address!("0x6bf1c0c174e11b933e7d8940afadf8bb7b8d421c");

        let is_module_installed = is_module_installed(
            eoa_validator_address,
            address,
            provider.clone(),
        )
        .await?;

        eyre::ensure!(is_module_installed, "Module is not installed");

        let call = {
            let target = address;
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

        let encoded_calls: Bytes = encode_calls(calls).into();
        let response = send_transaction(
            address,
            eoa_validator_address,
            entry_point_address,
            encoded_calls,
            bundler_client,
            provider.clone(),
            signer_private_key.to_string(),
        )
        .await?;

        dbg!(response);

        Ok(())
    }
}
