use alloy::sol;
use ciborium::{Value, de::from_reader};
use coset::{AsCborValue, CoseKey};
use eyre::Result;

sol! {
    #[derive(Debug)]
    struct PasskeySignature {
        bytes authenticator_data;
        bytes client_data_json;
        bytes32[2] signature;
    }

    #[derive(Debug)]
    struct PasskeyModuleParams {
        bytes signature;
        address validator;
        bytes[] validator_data;
    }
}

pub fn get_public_key_bytes_from_passkey_signature(
    public_passkey: &[u8],
) -> Result<([u8; 32], [u8; 32])> {
    let cbor_value: Value = from_reader(public_passkey)
        .map_err(|e| eyre::eyre!("Failed to parse CBOR: {}", e))?;

    let cose_key: CoseKey = CoseKey::from_cbor_value(cbor_value)
        .map_err(|e| eyre::eyre!("Failed to parse COSE key: {}", e))?;

    let params: Vec<(coset::Label, Value)> = cose_key.params;

    // COSE key parameter labels for EC2
    const EC2_X: i64 = -2; // x coordinate
    const EC2_Y: i64 = -3; // y coordinate

    let x: [u8; 32] = params
        .iter()
        .find(|(label, _)| matches!(label, coset::Label::Int(n) if *n == EC2_X))
        .and_then(|(_, value)| value.as_bytes())
        .ok_or(eyre::eyre!("Missing x coordinate"))?
        .to_vec()
        .as_slice()
        .try_into()
        .map_err(|_| eyre::eyre!("X coordinate must be 32 bytes"))?;

    let y: [u8; 32] = params
        .iter()
        .find(|(label, _)| matches!(label, coset::Label::Int(n) if *n == EC2_Y))
        .and_then(|(_, value)| value.as_bytes())
        .ok_or(eyre::eyre!("Missing y coordinate"))?
        .to_vec()
        .as_slice()
        .try_into()
        .map_err(|_| eyre::eyre!("Y coordinate must be 32 bytes"))?;

    Ok((x, y))
}

pub fn get_passkey_signature_from_public_key_bytes(
    coordinates: ([u8; 32], [u8; 32]),
) -> Result<Vec<u8>> {
    let (x, y) = coordinates;

    let cose_key = coset::CoseKeyBuilder::new_ec2_pub_key(
        coset::iana::EllipticCurve::P_256,
        x.to_vec(),
        y.to_vec(),
    )
    .build();

    let cbor_value = cose_key
        .to_cbor_value()
        .map_err(|e| eyre::eyre!("Failed to convert to CBOR: {}", e))?;

    let mut output = Vec::new();
    ciborium::ser::into_writer(&cbor_value, &mut output)
        .map_err(|e| eyre::eyre!("Failed to serialize CBOR: {}", e))?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eyre::Result;

    #[test]
    fn test_decode_cbor_cose_key() -> Result<()> {
        // Create sample CBOR-encoded COSE key with known x,y coordinates
        let sample_public_key = [
            0xa5, // map of 5 pairs
            0x01, // key 1 (kty)
            0x02, // value 2 (EC2)
            0x03, // key 3 (alg)
            0x26, // value -7 (ES256)
            0x20, // key -1 (crv)
            0x01, // value 1 (P-256)
            0x21, // key -2 (x coordinate)
            0x58, 0x20, // bytes(32)
        ]
        .iter()
        .chain([0x01; 32].iter())
        .chain(
            [
                0x22, // key -3 (y coordinate)
                0x58, 0x20, // bytes(32)
            ]
            .iter(),
        )
        .chain([0x02; 32].iter())
        .copied()
        .collect::<Vec<u8>>();

        let (x, y) =
            get_public_key_bytes_from_passkey_signature(&sample_public_key)?;

        assert!(x.iter().all(|&byte| byte == 0x01));
        assert!(y.iter().all(|&byte| byte == 0x02));

        Ok(())
    }

    #[test]
    fn test_passkey_roundtrip() -> Result<()> {
        let original_x = [0x01; 32];
        let original_y = [0x02; 32];

        let cose_bytes = get_passkey_signature_from_public_key_bytes((
            original_x, original_y,
        ))?;

        let (decoded_x, decoded_y) =
            get_public_key_bytes_from_passkey_signature(&cose_bytes)?;

        assert_eq!(original_x, decoded_x);
        assert_eq!(original_y, decoded_y);

        Ok(())
    }
}
