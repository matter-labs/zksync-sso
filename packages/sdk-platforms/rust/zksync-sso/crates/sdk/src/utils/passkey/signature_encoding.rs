use crate::{
    config::contracts::PasskeyContracts, utils::passkey::unwrap_signature,
};
use alloy::{
    dyn_abi::SolType,
    primitives::{Bytes, FixedBytes},
    sol,
};
use eyre::Result;
use hex;

type FatSignature = sol! { tuple(bytes, bytes, bytes32[2]) };
type FullSignature = sol! { tuple(bytes, address, bytes[]) };

pub fn encode_fat_signature(
    auth_data: Vec<u8>,
    client_data_json: Vec<u8>,
    unwrapped_sig: unwrap_signature::UnwrappedSignature,
    s: FixedBytes<32>,
) -> Result<Vec<u8>> {
    let encoded_fat_signature = FatSignature::abi_encode_params(&(
        auth_data,
        client_data_json,
        [
            <[u8; 32]>::try_from(unwrapped_sig.r.as_slice()).unwrap(),
            <[u8; 32]>::try_from(s.as_slice()).unwrap(),
        ],
    ));
    Ok(encoded_fat_signature)
}

pub fn encode_full_signature(
    encoded_fat_signature: Vec<u8>,
    contracts: &PasskeyContracts,
) -> Result<Vec<u8>> {
    encode_full_signature_with_validator_data(
        encoded_fat_signature,
        contracts,
        None,
    )
}

pub fn encode_full_signature_with_validator_data(
    encoded_fat_signature: Vec<u8>,
    contracts: &PasskeyContracts,
    validator_data: Option<Vec<Vec<u8>>>,
) -> Result<Vec<u8>> {
    println!("\n=== encode_full_signature_with_validator_data Function ===");
    println!("\nInput Parameters:");
    println!("Fat signature: 0x{}", hex::encode(&encoded_fat_signature));
    println!("Validator address: 0x{}", hex::encode(contracts.passkey));

    let validator_data =
        validator_data.unwrap_or_else(|| vec![Bytes::from(vec![]).to_vec()]);
    println!(
        "Validator data: {:?}",
        validator_data
            .iter()
            .map(|d| format!("0x{}", hex::encode(d)))
            .collect::<Vec<_>>()
    );

    let encoded: Vec<u8> = FullSignature::abi_encode_params(&(
        encoded_fat_signature,
        contracts.passkey,
        validator_data,
    ));

    let result = format!("0x{}", hex::encode(&encoded));
    println!("\nEncoded ABI Parameters:");
    println!("Result: {}", result);

    println!("\nEncoded Data Structure:");
    println!("- First 32 bytes (offset to fat signature): {}", &result[0..66]);
    println!("- Next 32 bytes (validator address): {}", &result[66..130]);
    println!(
        "- Next 32 bytes (offset to validator data array): {}",
        &result[130..194]
    );
    println!(
        "- Fat signature length: {}",
        usize::from_str_radix(result[194..258].trim_start_matches("0x"), 16)
            .unwrap_or(0)
    );
    println!("- Fat signature data starts at: {}", &result[258..322]);
    println!(
        "- Validator data array length: {}",
        usize::from_str_radix(
            result[result.len() - 128..result.len() - 64]
                .trim_start_matches("0x"),
            16
        )
        .unwrap_or(0)
    );

    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::contracts::PasskeyContracts, utils::passkey::unwrap_signature,
    };
    use alloy::primitives::{address, FixedBytes};
    use base64::Engine;
    use eyre::Result;
    use hex;

    #[test]
    fn test_encode_fat_signature() -> Result<()> {
        println!("\n=== encode_fat_signature Test ===");

        let auth_data_b64 =
            "08tFjuLOhgB6vt7kcKBTmkNjX9Yu4Wdm0LLy_MUj0v0dAAAAAA";
        println!("\nInput Parameters:");
        println!("authenticatorData: {}", auth_data_b64);
        let auth_data = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(auth_data_b64)?;
        println!("authenticatorData (decoded): {}", hex::encode(&auth_data));

        let client_data_json_b64 = "eyJ0eXBlIjoid2ViYXV0aG4uZ2V0IiwiY2hhbGxlbmdlIjoiODI3bm9uQVlEbXFnN3J2SFJFVURWOFhGZ3RveHhLZVhxdHJMcERram4zbyIsIm9yaWdpbiI6Imh0dHBzOi8vc29vLXNkay1leGFtcGxlLXBhZ2VzLnBhZ2VzLmRldiJ9";
        println!("\nclientDataJSON: {}", client_data_json_b64);
        let client_data_json = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(client_data_json_b64)?;
        println!(
            "clientDataJSON (decoded): {}",
            String::from_utf8_lossy(&client_data_json)
        );

        let r_hex =
            "1e6bd398700475910fb66389f177f6d4aec39230e20c29f019457c0867e30778";
        let s_hex =
            "2912824281822d4781ea9a513fdaade816234a7960363c47a0c9d7e469b85ff2";
        println!("\nSignature components:");
        println!("r: {}", r_hex);
        println!("s: {}", s_hex);

        let r = hex::decode(r_hex)?;
        let s = hex::decode(s_hex)?;

        let unwrapped_sig = unwrap_signature::UnwrappedSignature {
            r: FixedBytes::from_slice(&r),
            s: FixedBytes::from_slice(&s),
        };

        let s_bytes: FixedBytes<32> = s.as_slice().try_into()?;
        let result = encode_fat_signature(
            auth_data,
            client_data_json,
            unwrapped_sig,
            s_bytes,
        )?;
        let result_hex = format!("0x{}", hex::encode(&result));
        println!("\nFat signature result: {}", result_hex);

        let expected = "0x000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000e01e6bd398700475910fb66389f177f6d4aec39230e20c29f019457c0867e307782912824281822d4781ea9a513fdaade816234a7960363c47a0c9d7e469b85ff20000000000000000000000000000000000000000000000000000000000000025d3cb458ee2ce86007abedee470a0539a43635fd62ee16766d0b2f2fcc523d2fd1d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000847b2274797065223a22776562617574686e2e676574222c226368616c6c656e6765223a223832376e6f6e4159446d716737727648524555445638584667746f78784b65587174724c70446b6a6e336f222c226f726967696e223a2268747470733a2f2f736f6f2d73646b2d6578616d706c652d70616765732e70616765732e646576227d00000000000000000000000000000000000000000000000000000000";
        println!("\nExpected hex: {}", expected);

        println!("\nLength comparison:");
        println!("Result length: {}", result_hex.len());
        println!("Expected length: {}", expected.len());

        // Compare chunks
        println!("\nDetailed comparison:");
        let chunk_size = 64;
        for i in (0..result_hex.len()).step_by(chunk_size) {
            let result_chunk =
                &result_hex[i..std::cmp::min(i + chunk_size, result_hex.len())];
            let expected_chunk =
                &expected[i..std::cmp::min(i + chunk_size, expected.len())];
            if result_chunk != expected_chunk {
                println!("\nDifference at position {}:", i);
                println!("Result:   {}", result_chunk);
                println!("Expected: {}", expected_chunk);
            }
        }

        assert_eq!(result_hex, expected);
        Ok(())
    }

    #[test]
    fn test_encode_full_signature() -> Result<()> {
        let fat_signature = hex::decode("000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000e01e6bd398700475910fb66389f177f6d4aec39230e20c29f019457c0867e307782912824281822d4781ea9a513fdaade816234a7960363c47a0c9d7e469b85ff20000000000000000000000000000000000000000000000000000000000000025d3cb458ee2ce86007abedee470a0539a43635fd62ee16766d0b2f2fcc523d2fd1d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000847b2274797065223a22776562617574686e2e676574222c226368616c6c656e6765223a223832376e6f6e4159446d716737727648524555445638584667746f78784b65587174724c70446b6a6e336f222c226f726967696e223a2268747470733a2f2f736f6f2d73646b2d6578616d706c652d70616765732e70616765732e646576227d00000000000000000000000000000000000000000000000000000000")?;
        let contracts = PasskeyContracts {
            account_factory: address!(
                "0000000000000000000000000000000000000000"
            ),
            passkey: address!("1234567890123456789012345678901234567890"),
            session: address!("0000000000000000000000000000000000000000"),
            account_paymaster: address!(
                "0000000000000000000000000000000000000000"
            ),
        };

        let result = encode_full_signature(fat_signature, &contracts)?;

        assert_eq!(
            format!("0x{}", hex::encode(result)),
            "0x00000000000000000000000000000000000000000000000000000000000000600000000000000000000000001234567890123456789012345678901234567890000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000e01e6bd398700475910fb66389f177f6d4aec39230e20c29f019457c0867e307782912824281822d4781ea9a513fdaade816234a7960363c47a0c9d7e469b85ff20000000000000000000000000000000000000000000000000000000000000025d3cb458ee2ce86007abedee470a0539a43635fd62ee16766d0b2f2fcc523d2fd1d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000847b2274797065223a22776562617574686e2e676574222c226368616c6c656e6765223a223832376e6f6e4159446d716737727648524555445638584667746f78784b65587174724c70446b6a6e336f222c226f726967696e223a2268747470733a2f2f736f6f2d73646b2d6578616d706c652d70616765732e70616765732e646576227d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000"
        );

        Ok(())
    }

    #[test]
    fn test_encode_full_signature_custom_validator() -> Result<()> {
        println!("\n=== encode_full_signature Custom Validator Data Test ===");

        let fat_signature = hex::decode("0000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000e01e6bd398700475910fb66389f177f6d4aec39230e20c29f019457c0867e307782912824281822d4781ea9a513fdaade816234a7960363c47a0c9d7e469b85ff2000000000000000000000000000000000000000000000000000000000000002500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000847b2274797065223a22776562617574686e2e676574222c226368616c6c656e6765223a223832376e6f6e4159446d716737727648524555445638584667746f78784b65587174724c70446b6a6e336f222c226f726967696e223a2268747470733a2f2f736f6f2d73646b2d6578616d706c652d70616765732e70616765732e646576227d")?;
        println!("\nInput Parameters:");
        println!("Fat signature: 0x{}", hex::encode(&fat_signature));

        let contracts = PasskeyContracts {
            account_factory: address!(
                "0000000000000000000000000000000000000000"
            ),
            passkey: address!("1234567890123456789012345678901234567890"),
            session: address!("0000000000000000000000000000000000000000"),
            account_paymaster: address!(
                "0000000000000000000000000000000000000000"
            ),
        };
        println!("Validator address: 0x{}", hex::encode(contracts.passkey));

        let validator_data = vec![hex::decode("1234")?, hex::decode("5678")?];
        println!(
            "Validator data: {:?}",
            validator_data
                .iter()
                .map(|d| format!("0x{}", hex::encode(d)))
                .collect::<Vec<_>>()
        );

        let result = encode_full_signature_with_validator_data(
            fat_signature,
            &contracts,
            Some(validator_data),
        )?;
        let result_hex = format!("0x{}", hex::encode(&result));
        println!(
            "\nFull formatted signature with custom validator data: {}",
            result_hex
        );

        println!("\nVerifying validator data presence:");
        println!("Contains 0x1234: {}", result_hex.contains("1234"));
        println!("Contains 0x5678: {}", result_hex.contains("5678"));

        assert!(result_hex.contains("1234"));
        assert!(result_hex.contains("5678"));

        Ok(())
    }
}
