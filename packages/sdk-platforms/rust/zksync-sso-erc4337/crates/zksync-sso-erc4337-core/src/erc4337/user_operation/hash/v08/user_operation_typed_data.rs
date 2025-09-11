use crate::erc4337::entry_point::PackedUserOperation;
use alloy::{
    primitives::{Address, FixedBytes},
    signers::{Signature, SignerSync, local::PrivateKeySigner},
    sol_types::{Eip712Domain, SolStruct, eip712_domain},
};

pub mod constants;
pub mod hash_typed_data;
pub mod user_operation_hash;

pub(crate) fn create_domain(
    chain_id: u64,
    entry_point_address: Address,
) -> Eip712Domain {
    eip712_domain! {
        name: "ERC4337",
        version: "1",
        chain_id: chain_id,
        verifying_contract: entry_point_address,
    }
}

pub fn signing_hash(
    chain_id: u64,
    entry_point_address: Address,
    user_operation: PackedUserOperation,
) -> eyre::Result<FixedBytes<32>> {
    println!("Chain ID: {:?}", chain_id);
    print!("Entry Point Address: {:?}", entry_point_address);
    println!("User Operation: {:?}", user_operation);
    let domain = create_domain(chain_id, entry_point_address);
    println!("Domain: {:?}", domain);
    let hash = user_operation.eip712_signing_hash(&domain);
    println!("Hash: {:?}", hash);
    Ok(hash)
}

pub fn sign_user_operation(
    chain_id: u64,
    entry_point_address: Address,
    user_operation: PackedUserOperation,
    signer: PrivateKeySigner,
) -> eyre::Result<Signature> {
    let hash = signing_hash(chain_id, entry_point_address, user_operation)?;
    let signature = signer.sign_hash_sync(&hash)?;
    Ok(signature)
}
