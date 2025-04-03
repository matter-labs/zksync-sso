use alloy::primitives::{keccak256, Address, Bytes, FixedBytes};

// TODO: consider uptreaming this to the `alloy-zksync` crate
pub fn create2_address(
    account_factory_address: Address,
    smart_account_bytecode_hash: FixedBytes<32>,
    account_id: FixedBytes<32>,
    smart_account_proxy_address: Bytes,
) -> Address {
    let prefix = keccak256(b"zksyncCreate2");

    let mut padded_sender = [0u8; 32];
    padded_sender[12..].copy_from_slice(account_factory_address.as_slice());

    let salt = account_id.as_slice();

    let bytecode_hash = smart_account_bytecode_hash.as_slice();

    let input_hash = keccak256(smart_account_proxy_address);

    let mut combined = [0u8; 160];
    combined[0..32].copy_from_slice(prefix.as_slice());
    combined[32..64].copy_from_slice(&padded_sender);
    combined[64..96].copy_from_slice(salt);
    combined[96..128].copy_from_slice(bytecode_hash);
    combined[128..160].copy_from_slice(input_hash.as_slice());

    let hash = keccak256(combined);
    Address::from_slice(&hash[12..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, bytes};

    #[test]
    fn test_create2_address() {
        // Arrange
        let factory = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
        let bytecode_hash = FixedBytes::from([1u8; 32]);
        let account_id = FixedBytes::from([1u8; 32]);
        let proxy_address = bytes!("0123456789");

        // Act
        let result = create2_address(
            factory,
            bytecode_hash,
            account_id,
            proxy_address.clone(),
        );

        // Assert
        // 1. Result should be a valid address (20 bytes)
        assert_eq!(result.as_slice().len(), 20);

        // 2. Test determinism - same inputs should give same address
        let result2 = create2_address(
            factory,
            bytecode_hash,
            account_id,
            proxy_address.clone(),
        );
        assert_eq!(result, result2);

        // 3. Different inputs should give different addresses
        let different_factory =
            address!("2222222222222222222222222222222222222222");
        let different_result = create2_address(
            different_factory,
            bytecode_hash,
            account_id,
            proxy_address.clone(),
        );
        assert_ne!(result, different_result);

        // 4. Verify the components are correctly included in the hash
        let mut data = Vec::with_capacity(160);
        data.extend_from_slice(keccak256(b"zksyncCreate2").as_slice());
        let mut padded_factory = [0u8; 32];
        padded_factory[12..].copy_from_slice(factory.as_slice());
        data.extend_from_slice(&padded_factory);
        data.extend_from_slice(account_id.as_slice());
        data.extend_from_slice(bytecode_hash.as_slice());
        data.extend_from_slice(keccak256(proxy_address).as_slice());

        let expected = Address::from_slice(&keccak256(data)[12..]);
        assert_eq!(result, expected);
    }
}
