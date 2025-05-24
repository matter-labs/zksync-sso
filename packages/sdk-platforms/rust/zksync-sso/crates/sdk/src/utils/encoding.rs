use crate::utils::passkey::signature_encoding::base64_url_to_uint8_array;
use alloy::{
    primitives::{Address, Bytes, FixedBytes},
    sol,
    sol_types::SolType,
};
use eyre::Result;
use log::debug;

pub mod paymaster;

sol! {
    struct PasskeyParams {
        bytes credentialId;
        bytes32[2] xyPublicKeys;
        string expectedOrigin;
    }
}

#[derive(Debug, Clone)]
pub struct PasskeyModuleParams {
    pub passkey_id: String,
    pub passkey_public_key: ([u8; 32], [u8; 32]),
    pub expected_origin: String,
}

pub fn encode_passkey_module_parameters(
    passkey: PasskeyModuleParams,
) -> Result<Bytes> {
    debug!(
        "XDB encode_passkey_module_parameters - passkey_id: {:?}",
        passkey.passkey_id
    );
    debug!(
        "XDB encode_passkey_module_parameters - passkey_public_key: {:?}",
        passkey.passkey_public_key
    );
    debug!(
        "XDB encode_passkey_module_parameters - expected_origin: {:?}",
        passkey.expected_origin
    );

    let credential_id =
        base64_url_to_uint8_array(passkey.passkey_id.clone(), true)?;

    debug!(
        "XDB encode_passkey_module_parameters - credential_id: {:?}",
        credential_id
    );

    let x = FixedBytes::from_slice(&passkey.passkey_public_key.0);
    let y = FixedBytes::from_slice(&passkey.passkey_public_key.1);

    let params = PasskeyParams {
        credentialId: credential_id.into(),
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
    use alloy::primitives::{address, hex};

    #[test]
    fn test_encode_passkey_module_parameters() -> eyre::Result<()> {
        // Arrange
        let x = hex::decode(
            "1234567890123456789012345678901234567890123456789012345678901234",
        )?
        .try_into()
        .map_err(|e| {
            eyre::eyre!("X coordinate must be 32 bytes, error: {:?}", e)
        })?;

        let y = hex::decode(
            "abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd",
        )?
        .try_into()
        .map_err(|e| {
            eyre::eyre!("Y coordinate must be 32 bytes, error: {:?}", e)
        })?;

        let passkey = PasskeyModuleParams {
            passkey_id: "unique-base64encoded-string".to_string(),
            passkey_public_key: (x, y),
            expected_origin: String::from("https://example.com"),
        };

        // Act
        let result = encode_passkey_module_parameters(passkey.clone())?;

        // Assert
        let hex_result = format!("0x{}", hex::encode(&result));
        let decoded_credential_id = {
            use base64::Engine;
            let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;

            engine.decode(passkey.passkey_id)?
        };
        eyre::ensure!(
            hex_result.contains(&hex::encode(x)),
            "X coordinate not found in result"
        );
        eyre::ensure!(
            hex_result.contains(&hex::encode(y)),
            "Y coordinate not found in result"
        );
        eyre::ensure!(
            hex_result
                .contains(&hex::encode(passkey.expected_origin.as_bytes())),
            "Expected origin not found in result"
        );
        eyre::ensure!(
            hex_result.contains(&hex::encode(decoded_credential_id)),
            "Credential ID not found in result"
        );
        eyre::ensure!(
            hex_result
                == "0x00000000000000000000000000000000000000000000000000000000000000801234567890123456789012345678901234567890123456789012345678901234abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd00000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000014ba78aab9ef9b6ac7bae1e9dca1d79dfacb6b8a78000000000000000000000000000000000000000000000000000000000000000000000000000000000000001368747470733a2f2f6578616d706c652e636f6d00000000000000000000000000"
        );

        Ok(())
    }

    #[test]
    fn test_encode_module_data() {
        // Arrange
        let address = address!("1234567890123456789012345678901234567890");
        let parameters = hex::decode("abcdef").unwrap().into();
        let module_data = ModuleData { address, parameters };
        let expected_result = "0x000000000000000000000000123456789012345678901234567890123456789000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000003abcdef0000000000000000000000000000000000000000000000000000000000".to_string();

        // Act
        let result = encode_module_data(module_data).unwrap();

        // Assert
        let hex_result = format!("0x{}", hex::encode(&result));
        assert_eq!(hex_result, expected_result);
    }
}
