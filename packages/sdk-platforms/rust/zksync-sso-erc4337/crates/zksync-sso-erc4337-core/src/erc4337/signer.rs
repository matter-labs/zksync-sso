use crate::erc4337::{
    entry_point::PackedUserOperation,
    user_operation::wrapper_v07::PackedUserOperationWrapperV07,
};
use alloy::{
    hex,
    primitives::{Address, Bytes, FixedBytes},
    signers::{SignerSync, local::PrivateKeySigner},
};
use eyre;
use std::sync::Arc;

pub type SignatureProvider =
    Arc<dyn Fn(FixedBytes<32>) -> eyre::Result<Bytes> + Send + Sync>;

#[derive(Clone)]
pub struct Signer {
    pub stub_signature: Bytes,
    pub provider: SignatureProvider,
}

pub fn sign_user_operation_v07_with_ecdsa(
    uo: &PackedUserOperation,
    ep: &Address,
    chain_id: u64,
    signer: PrivateKeySigner,
) -> eyre::Result<PackedUserOperation> {
    let uo_wrapper = PackedUserOperationWrapperV07::from(uo.clone());
    let hash = uo_wrapper.hash(ep, chain_id);

    println!("hash: {:?}", hash.clone());

    let message = hash.0;

    println!("message: {:?}", message.clone());

    let message_bytes = message.to_vec();

    println!("message_bytes: {:?}", message_bytes.clone());

    let signature = signer.sign_message_sync(&message_bytes)?;
    println!("signature: {:?}", signature);
    let sig_vec: Vec<u8> = signature.into();
    println!("hex::encode(sig_vec): {:?}", hex::encode(sig_vec.clone()));

    let mut user_operation = uo.clone();
    user_operation.signature = sig_vec.into();
    Ok(user_operation)
}
