use alloy::{
    primitives::{FixedBytes, hex},
    sol_types::{Eip712Domain, SolStruct},
};
use alloy_dyn_abi::TypedData;

pub fn dyn_hash_typed_data(
    typed_data: TypedData,
) -> eyre::Result<FixedBytes<32>> {
    let hash = typed_data.eip712_signing_hash()?;
    println!("Hash: {}", hex::encode(hash));
    Ok(hash)
}

pub fn hash_typed_data<T: SolStruct>(
    ty: T,
    domain: Eip712Domain,
) -> FixedBytes<32> {
    let hash = ty.eip712_signing_hash(&domain);
    println!("Hash: {}", hex::encode(hash));
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::user_operation::hash::v08::user_operation_typed_data::constants;

    #[test]
    fn test_hash_typed_data() -> eyre::Result<()> {
        let payload = constants::basic();
        let message = payload.message;
        let domain = payload.domain;

        let typed_data = TypedData::from_struct(&message, Some(domain.clone()));

        let hashed_type_data_dyn = dyn_hash_typed_data(typed_data)?;

        println!("Hashed Type Data Dyn: {}", hex::encode(hashed_type_data_dyn));

        let hashed_typed_data = hash_typed_data(message, domain);

        println!("Hashed Typed Data: {}", hex::encode(hashed_typed_data));

        eyre::ensure!(
            hashed_type_data_dyn == hashed_typed_data,
            "hashing functions should return identical data, dyn: {}, static: {}",
            hex::encode(hashed_type_data_dyn),
            hex::encode(hashed_typed_data)
        );

        Ok(())
    }
}
