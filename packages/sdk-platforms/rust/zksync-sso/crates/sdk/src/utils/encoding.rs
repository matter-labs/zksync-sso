use alloy::primitives::{Address, Bytes, FixedBytes};
use alloy::sol;
use alloy::sol_types::SolType;
use eyre::Result;

pub mod paymaster;

pub struct PasskeyModuleParams {
    pub passkey_public_key: ([u8; 32], [u8; 32]),
    pub expected_origin: String,
}

pub fn encode_passkey_module_parameters(
    passkey: PasskeyModuleParams,
) -> Result<Bytes> {
    sol! {
        struct PasskeyParams {
            bytes32[2] xyPublicKeys;
            string expectedOrigin;
        }
    }

    let x = FixedBytes::from_slice(&passkey.passkey_public_key.0);
    let y = FixedBytes::from_slice(&passkey.passkey_public_key.1);

    let params = PasskeyParams {
        xyPublicKeys: [x, y],
        expectedOrigin: passkey.expected_origin,
    };

    let params_bytes = PasskeyParams::abi_encode_params(&params);

    Ok(params_bytes.into())
}

pub struct ModuleData {
    pub address: Address,
    pub parameters: Bytes,
}

pub fn encode_module_data(module_data: ModuleData) -> Result<Bytes> {
    sol! {
        struct ModuleParams {
            address module_address;
            bytes parameters;
        }
    }

    let params = ModuleParams {
        module_address: module_data.address,
        parameters: module_data.parameters,
    };

    let encoded = ModuleParams::abi_encode_params(&params);

    Ok(encoded.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{hex, address};

    #[test]
    fn test_encode_passkey_module_parameters() {
        // Arrange
        let x = hex::decode(
            "1234567890123456789012345678901234567890123456789012345678901234",
        )
        .unwrap()
        .try_into()
        .unwrap();
        let y = hex::decode(
            "abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd",
        )
        .unwrap()
        .try_into()
        .unwrap();
        let passkey = PasskeyModuleParams {
            passkey_public_key: (x, y),
            expected_origin: String::from("https://example.com"),
        };

        // Act
        let result = encode_passkey_module_parameters(passkey).unwrap();

        // Assert
        let hex_result = format!("0x{}", hex::encode(&result));
        assert_eq!(
            hex_result,
            "0x1234567890123456789012345678901234567890123456789012345678901234abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000001368747470733a2f2f6578616d706c652e636f6d00000000000000000000000000"
        );
    }

    #[test]
    fn test_encode_module_data() {
        // Arrange
        let address = address!("1234567890123456789012345678901234567890");
        let parameters = hex::decode("abcdef").unwrap().into();
        let module_data = ModuleData { address, parameters };
        let expected_result = 
            "0x000000000000000000000000123456789012345678901234567890123456789000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000003abcdef0000000000000000000000000000000000000000000000000000000000".to_string()
        ;

        // Act
        let result = encode_module_data(module_data).unwrap();

        // Assert
        let hex_result = format!("0x{}", hex::encode(&result));
        assert_eq!(hex_result, expected_result);
    }
}
