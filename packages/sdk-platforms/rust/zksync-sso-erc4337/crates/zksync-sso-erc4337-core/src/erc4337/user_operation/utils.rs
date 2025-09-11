use alloy::primitives::{Address, Bytes, FixedBytes, U256};

/// Pack two U256 values into a 32-byte array, taking the lower 16 bytes from each
/// This is used for packing gas limits and fees according to ERC-4337 specification
pub fn pack_u256_pair(first: U256, second: U256) -> FixedBytes<32> {
    let mut result = [0u8; 32];

    // Convert U256 to bytes and take the lower 16 bytes from each
    let first_bytes = first.to_be_bytes_vec();
    let second_bytes = second.to_be_bytes_vec();

    // Take last 16 bytes from each (lower 128 bits)
    result[0..16].copy_from_slice(&first_bytes[16..32]);
    result[16..32].copy_from_slice(&second_bytes[16..32]);

    FixedBytes::from(result)
}

/// Concatenate factory address and factory data to create init code
pub fn concat_factory_data(factory: Address, factory_data: &Bytes) -> Bytes {
    let mut result = Vec::with_capacity(20 + factory_data.len());
    result.extend_from_slice(factory.as_slice());
    result.extend_from_slice(factory_data.as_ref());
    Bytes::from(result)
}

/// Concatenate paymaster address with gas limits and data
pub fn concat_paymaster_data(
    paymaster: Address,
    verification_gas_limit: U256,
    post_op_gas_limit: U256,
    paymaster_data: &Bytes,
) -> Bytes {
    let mut result = Vec::with_capacity(20 + 16 + 16 + paymaster_data.len());

    // Add paymaster address (20 bytes)
    result.extend_from_slice(paymaster.as_slice());

    // Add verification gas limit (16 bytes, padded)
    let verification_bytes = verification_gas_limit.to_be_bytes_vec();
    result.extend_from_slice(&verification_bytes[16..32]);

    // Add post-op gas limit (16 bytes, padded)
    let post_op_bytes = post_op_gas_limit.to_be_bytes_vec();
    result.extend_from_slice(&post_op_bytes[16..32]);

    // Add paymaster data
    result.extend_from_slice(paymaster_data.as_ref());

    Bytes::from(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, b256, bytes};

    #[test]
    fn test_pack_u256_pair() {
        let first = U256::from(0x1000);
        let second = U256::from(0x2000);
        let packed = pack_u256_pair(first, second);

        // The result should have lower 16 bytes of first, then lower 16 bytes of second
        let expected = b256!(
            "0000000000000000000000000000100000000000000000000000000000002000"
        );
        assert_eq!(packed, expected);
    }

    #[test]
    fn test_concat_factory_data() {
        let factory = address!("1234567890123456789012345678901234567890");
        let factory_data = bytes!("deadbeef");
        let result = concat_factory_data(factory, &factory_data);

        let expected =
            bytes!("1234567890123456789012345678901234567890deadbeef");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_concat_paymaster_data() {
        let paymaster = address!("1234567890123456789012345678901234567890");
        let verification_gas_limit = U256::from(0x1000);
        let post_op_gas_limit = U256::from(0x2000);
        let paymaster_data = bytes!("abcdef");

        let result = concat_paymaster_data(
            paymaster,
            verification_gas_limit,
            post_op_gas_limit,
            &paymaster_data,
        );

        // paymaster (20) + verification_gas_limit (16) + post_op_gas_limit (16) + data
        let expected = bytes!(
            "12345678901234567890123456789012345678900000000000000000000000000000100000000000000000000000000000002000abcdef"
        );
        assert_eq!(result, expected);
    }
}
