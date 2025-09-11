use super::user_operation::wrapper_v07::create_user_operation;
use crate::{
    config::Config,
    erc4337::{entry_point::PackedUserOperation, send_call::SendCalls},
};
use alloy::{primitives::Bytes, signers::local::PrivateKeySigner};

pub struct SignedUserOperation {
    pub packed: PackedUserOperation,
    pub user_op_hash: Bytes,
}

pub fn sign_user_operation_dummy(
    _config: &Config,
    _signer: &PrivateKeySigner,
    _op: SendCalls,
) -> SignedUserOperation {
    SignedUserOperation {
        packed: create_user_operation(),
        user_op_hash: Bytes::from_static(b"dummy_user_op_hash"),
    }
}
