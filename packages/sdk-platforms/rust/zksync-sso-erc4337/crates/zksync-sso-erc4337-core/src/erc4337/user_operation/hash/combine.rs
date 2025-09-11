use alloy::primitives::{B256, U256};

pub fn combine_and_trim_first_16_bytes(item1: U256, item2: U256) -> B256 {
    let mut vec = Vec::with_capacity(32);
    vec.extend_from_slice(&item1.to_be_bytes_vec()[16..]);
    vec.extend_from_slice(&item2.to_be_bytes_vec()[16..]);
    B256::from_slice(&vec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::b256;

    #[test]
    fn test_combine_and_trim_first_16_bytes() {
        let expected_result = b256!(
            "0000000000000000000000000000000100000000000000000000000000000002"
        );
        let item1 = U256::from(1);
        let item2 = U256::from(2);
        let result = combine_and_trim_first_16_bytes(item1, item2);
        assert_eq!(result, expected_result);
    }
}
