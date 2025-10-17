use crate::erc4337::{
    account::modular_smart_account::{
        nonce::get_nonce,
        signature::{eoa_signature, stub_signature_eoa},
    },
    bundler::{Bundler, pimlico::client::BundlerClient},
    entry_point::EntryPoint,
    signer::Signer,
    user_operation::hash::v08::get_user_operation_hash_entry_point,
};
use alloy::{
    primitives::{Address, Bytes, FixedBytes, U256, Uint},
    providers::Provider,
    rpc::types::erc4337::PackedUserOperation as AlloyPackedUserOperation,
};
use std::sync::Arc;

pub mod passkey;

pub async fn send_transaction_eoa<P: Provider + Send + Sync + Clone>(
    account: Address,
    eoa_validator: Address,
    entry_point: Address,
    call_data: Bytes,
    bundler_client: BundlerClient,
    provider: P,
    private_key_hex: String,
) -> eyre::Result<()> {
    let stub_sig = stub_signature_eoa(eoa_validator)?;

    let signature_provider = Arc::new(move |hash: FixedBytes<32>| {
        eoa_signature(&private_key_hex, eoa_validator, hash)
    });

    let signer =
        Signer { stub_signature: stub_sig, provider: signature_provider };

    send_transaction(
        account,
        entry_point,
        call_data,
        None,
        bundler_client,
        provider,
        signer,
    )
    .await
}

pub async fn send_transaction<P: Provider + Send + Sync + Clone>(
    account: Address,
    entry_point: Address,
    call_data: Bytes,
    nonce_key: Option<Uint<192, 3>>,
    bundler_client: BundlerClient,
    provider: P,
    signer: Signer,
) -> eyre::Result<()> {
    let nonce_key = nonce_key.unwrap_or_else(|| Uint::from(0));

    let nonce = get_nonce(entry_point, account, nonce_key, &provider).await?;

    let (estimated_gas, mut user_op) = {
        let alloy_user_op = {
            let stub_sig = signer.stub_signature;
            AlloyPackedUserOperation {
                sender: account,
                nonce,
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
                call_data,
                signature: stub_sig,
            }
        };

        let estimated_gas = bundler_client
            .estimate_user_operation_gas(&alloy_user_op, &entry_point)
            .await?;

        (estimated_gas, alloy_user_op)
    };

    user_op.call_gas_limit = estimated_gas.call_gas_limit;
    user_op.verification_gas_limit =
        (estimated_gas.verification_gas_limit * U256::from(6)) / U256::from(5);
    user_op.pre_verification_gas = estimated_gas.pre_verification_gas;

    user_op.max_priority_fee_per_gas = U256::from(0x77359400);
    user_op.max_fee_per_gas = U256::from(0x82e08afeu64);

    let packed_gas_limits: U256 =
        (user_op.verification_gas_limit << 128) | user_op.call_gas_limit;

    let gas_fees: U256 =
        (user_op.max_priority_fee_per_gas << 128) | user_op.max_fee_per_gas;

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

    let hash = get_user_operation_hash_entry_point(
        &packed_user_op,
        &entry_point,
        provider,
    )
    .await?;

    let signature_provider = signer.provider;
    let signature = signature_provider(hash.0)?;
    user_op.signature = signature;

    let user_op_hash =
        bundler_client.send_user_operation(entry_point, user_op).await?;

    bundler_client.wait_for_user_operation_receipt(user_op_hash).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::account::{
        erc7579::{
            Execution, calls::encode_calls,
            module_installed::is_module_installed,
        },
        modular_smart_account::deploy::{EOASigners, deploy_account},
    };
    use alloy::{
        primitives::{Bytes, U256, address},
        providers::ProviderBuilder,
        rpc::types::TransactionRequest,
        signers::local::PrivateKeySigner,
    };
    use std::str::FromStr;

    #[tokio::test]
    #[ignore = "needs local infrastructure to be running"]
    async fn test_send_transaction_contracts() -> eyre::Result<()> {
        let rpc_url = "http://localhost:8545".parse()?;

        let factory_address =
            address!("0x679FFF51F11C3f6CaC9F2243f9D14Cb1255F65A3");

        let entry_point_address =
            address!("0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108");

        let eoa_validator_address =
            address!("0x00427eDF0c3c3bd42188ab4C907759942Abebd93");

        let signer_private_key = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";

        let provider = {
            let signer = PrivateKeySigner::from_str(signer_private_key)?;
            let alloy_signer = signer.clone();
            let ethereum_wallet =
                alloy::network::EthereumWallet::new(alloy_signer.clone());

            ProviderBuilder::new()
                .wallet(ethereum_wallet.clone())
                .connect_http(rpc_url)
        };

        let signers =
            vec![address!("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")];

        let eoa_signers = EOASigners {
            addresses: signers,
            validator_address: eoa_validator_address,
        };

        let address = deploy_account(
            factory_address,
            Some(eoa_signers),
            None,
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

        send_transaction_eoa(
            address,
            eoa_validator_address,
            entry_point_address,
            encoded_calls,
            bundler_client,
            provider.clone(),
            signer_private_key.to_string(),
        )
        .await?;

        Ok(())
    }
}
