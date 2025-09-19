use alloy::{
    primitives::{B256, keccak256},
    rpc::types::erc4337::PackedUserOperation,
};

pub fn get_hashed_init_code(user_operation: &PackedUserOperation) -> B256 {
    let uo = user_operation.clone();
    let value_vec = if let (Some(factory), Some(factory_data)) =
        (uo.factory, uo.factory_data.clone())
    {
        let factory_vec: Vec<u8> = factory.to_vec();
        let factory_data_vec: Vec<u8> = factory_data.into();
        let mut bytes_vec: Vec<u8> = vec![];
        bytes_vec.extend(factory_vec);
        bytes_vec.extend(factory_data_vec);
        bytes_vec
    } else {
        let bytes_vec: Vec<u8> = vec![];
        bytes_vec
    };

    keccak256(value_vec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::user_operation::wrapper::PackedUserOperationWrapper;

    #[test]
    fn test_get_hashed_init_code() {
        let expected_hashed_init_code_hex = "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470";
        let user_operation = PackedUserOperationWrapper::mock().0;
        let hashed_init_code = get_hashed_init_code(&user_operation);
        assert_eq!(hashed_init_code.to_string(), expected_hashed_init_code_hex);
    }
}
