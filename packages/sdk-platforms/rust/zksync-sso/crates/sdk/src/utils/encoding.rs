use alloy::{
    primitives::{Address, Bytes},
    sol,
    sol_types::SolType,
};
use eyre::Result;

pub mod passkey;
pub mod paymaster;
pub mod session;

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

    let encoded = <ModuleParams as SolType>::abi_encode_params(&params);

    Ok(encoded.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, hex};

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
