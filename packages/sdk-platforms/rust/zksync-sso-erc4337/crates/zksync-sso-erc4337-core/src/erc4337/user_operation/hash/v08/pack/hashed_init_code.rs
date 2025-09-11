use crate::erc4337::{
    entry_point::PackedUserOperation,
    user_operation::hash::v08::pack::CodeReader,
};
use alloy::primitives::{Address, B256, keccak256};

pub fn get_hashed_init_code(user_operation: &PackedUserOperation) -> B256 {
    keccak256(&user_operation.initCode)
}

fn is_eip7702_init_code(init_code: &[u8]) -> bool {
    if init_code.len() < 2 {
        return false;
    }
    init_code[0] == 0x77 && init_code[1] == 0x02
}

fn eip7702_delegate_from_code(code: &[u8]) -> Option<Address> {
    if code.len() < 23 {
        return None;
    }
    if !(code[0] == 0xef && code[1] == 0x01 && code[2] == 0x00) {
        return None;
    }
    Some(Address::from_slice(&code[3..23]))
}

pub fn get_hashed_init_code_with_7702(
    user_operation: &PackedUserOperation,
    code_reader: &dyn CodeReader,
) -> B256 {
    let init = &user_operation.initCode;
    if !is_eip7702_init_code(init) {
        return keccak256(init);
    }
    if let Some(code) = code_reader.get_code(&user_operation.sender)
        && let Some(delegate) = eip7702_delegate_from_code(&code)
    {
        if init.len() <= 20 {
            return keccak256(delegate.as_slice());
        }
        let mut buf = Vec::with_capacity(20 + init.len().saturating_sub(20));
        buf.extend_from_slice(delegate.as_slice());
        buf.extend_from_slice(&init[20..]);
        return keccak256(buf);
    }
    keccak256(init)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc4337::user_operation::hash::v08::PackedUserOperationWrapper;
    use alloy::primitives::Address;

    struct MockCodeReader;
    impl super::super::CodeReader for MockCodeReader {
        fn get_code(&self, _address: &Address) -> Option<Vec<u8>> {
            None
        }
    }

    #[test]
    fn test_get_hashed_init_code() {
        let expected_hashed_init_code_hex = "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470";
        let user_operation = PackedUserOperationWrapper::mock().0;
        let code_reader = MockCodeReader;
        let hashed_init_code =
            get_hashed_init_code_with_7702(&user_operation, &code_reader);
        assert_eq!(hashed_init_code.to_string(), expected_hashed_init_code_hex);
    }
}
