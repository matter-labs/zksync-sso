pub mod eoa;
pub mod passkey;

use crate::erc4337::{
    bundler::{
        Bundler, models::receipt::UserOperationReceipt,
        pimlico::client::BundlerClient,
    },
    entry_point::{
        contract::EntryPoint,
        nonce::{GetNonceWithKeyParams, get_nonce_with_key},
    },
    paymaster::params::{PaymasterParams, build_paymaster_and_data},
    signer::Signer,
    user_operation::hash::user_operation_hash::get_user_operation_hash_entry_point,
};
use alloy::{
    primitives::{Address, Bytes, U256, Uint},
    providers::Provider,
    rpc::types::erc4337::PackedUserOperation as AlloyPackedUserOperation,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct FactoryPayload {
    pub factory: Address,
    pub factory_data: Option<Bytes>,
}

#[derive(Clone)]
pub struct SendUserOpParams<P: Provider + Send + Sync + Clone> {
    pub account: Address,
    pub entry_point: Address,
    pub factory_payload: Option<FactoryPayload>,
    pub call_data: Bytes,
    pub nonce_key: Option<Uint<192, 3>>,
    pub paymaster: Option<PaymasterParams>,
    pub bundler_client: BundlerClient,
    pub provider: P,
    pub signer: Signer,
}

pub async fn send_user_op<P>(
    params: SendUserOpParams<P>,
) -> eyre::Result<UserOperationReceipt>
where
    P: Provider + Send + Sync + Clone,
{
    let SendUserOpParams {
        account,
        entry_point,
        factory_payload,
        call_data,
        nonce_key,
        paymaster,
        bundler_client,
        provider,
        signer,
    } = params;

    let nonce_key = nonce_key.unwrap_or_else(|| Uint::from(0));

    let nonce = get_nonce_with_key(GetNonceWithKeyParams {
        sender: account,
        entry_point,
        key: nonce_key,
        provider: provider.clone(),
    })
    .await?;

    let (estimated_gas, mut user_op) = {
        let alloy_user_op = {
            let stub_sig = signer.stub_signature.clone();
            AlloyPackedUserOperation {
                sender: account,
                nonce,
                factory: factory_payload
                    .as_ref()
                    .map(|payload| payload.factory),
                factory_data: factory_payload
                    .as_ref()
                    .and_then(|payload| payload.factory_data.clone()),
                paymaster: paymaster.as_ref().map(|params| params.address),
                paymaster_verification_gas_limit: paymaster
                    .as_ref()
                    .and_then(|params| params.verification_gas_limit),
                paymaster_data: paymaster
                    .as_ref()
                    .map(|params| params.data.clone()),
                paymaster_post_op_gas_limit: paymaster
                    .as_ref()
                    .and_then(|params| params.post_op_gas_limit),
                call_gas_limit: Default::default(),
                max_priority_fee_per_gas: Default::default(),
                max_fee_per_gas: Default::default(),
                pre_verification_gas: Default::default(),
                verification_gas_limit: Default::default(),
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

    if let Some(paymaster) = paymaster.as_ref() {
        user_op.paymaster = Some(paymaster.address);
        user_op.paymaster_data = Some(paymaster.data.clone());
        user_op.paymaster_verification_gas_limit = estimated_gas
            .paymaster_verification_gas_limit
            .or(paymaster.verification_gas_limit);
        user_op.paymaster_post_op_gas_limit = estimated_gas
            .paymaster_post_op_gas_limit
            .or(paymaster.post_op_gas_limit);
    }

    let packed_gas_limits: U256 =
        (user_op.verification_gas_limit << 128) | user_op.call_gas_limit;

    let gas_fees: U256 =
        (user_op.max_priority_fee_per_gas << 128) | user_op.max_fee_per_gas;

    let init_code = if let Some(payload) = factory_payload.as_ref() {
        let address_bytes = payload.factory.as_slice();
        let data_len =
            payload.factory_data.as_ref().map(|data| data.len()).unwrap_or(0);
        let mut bytes = Vec::with_capacity(address_bytes.len() + data_len);
        bytes.extend_from_slice(address_bytes);
        if let Some(data) = payload.factory_data.as_ref() {
            bytes.extend_from_slice(data.as_ref());
        }
        Bytes::from(bytes)
    } else {
        Bytes::default()
    };

    let packed_user_op = EntryPoint::PackedUserOperation {
        sender: user_op.sender,
        nonce: user_op.nonce,
        initCode: init_code,
        callData: user_op.call_data.clone(),
        accountGasLimits: packed_gas_limits.to_be_bytes().into(),
        preVerificationGas: user_op.pre_verification_gas,
        gasFees: gas_fees.to_be_bytes().into(),
        paymasterAndData: build_paymaster_and_data(
            user_op.paymaster,
            user_op.paymaster_verification_gas_limit,
            user_op.paymaster_post_op_gas_limit,
            user_op.paymaster_data.as_ref(),
        ),
        signature: user_op.signature.clone(),
    };

    let hash = get_user_operation_hash_entry_point(
        &packed_user_op,
        &entry_point,
        provider.clone(),
    )
    .await?;

    let signature_provider = Arc::clone(&signer.provider);
    let signature = signature_provider(hash.0).await?;
    user_op.signature = signature;

    let user_op_hash =
        bundler_client.send_user_operation(entry_point, user_op).await?;

    let receipt =
        bundler_client.wait_for_user_operation_receipt(user_op_hash).await?;

    if !receipt.success {
        return Err(eyre::eyre!("User operation execution failed"));
    }

    Ok(receipt)
}
