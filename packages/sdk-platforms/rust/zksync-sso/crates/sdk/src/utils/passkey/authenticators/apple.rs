use ciborium::Value;
use eyre::Result;
use hex;
use log::debug;

pub mod verify;

#[allow(dead_code)]
#[derive(Debug)]
pub struct AttestationObject {
    pub auth_data: AuthenticatorData,
    #[allow(dead_code)]
    pub fmt: String,
    #[allow(dead_code)]
    pub att_stmt: AttestationStatement,
}

#[derive(Debug)]
pub struct AuthenticatorData {
    #[allow(dead_code)]
    pub rp_id_hash: [u8; 32],
    #[allow(dead_code)]
    pub flags: u8,
    #[allow(dead_code)]
    pub counter: u32,
    pub attested_data: Option<AttestedCredentialData>,
}

#[derive(Debug)]
pub struct AttestedCredentialData {
    #[allow(dead_code)]
    pub aaguid: [u8; 16],
    #[allow(dead_code)]
    pub credential_id: Vec<u8>,
    pub cose_key: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct AttestationStatement {
    #[allow(dead_code)]
    pub x5c: Vec<Vec<u8>>,
}

pub fn extract_public_key(
    raw_attestation: &[u8],
) -> Result<([u8; 32], [u8; 32])> {
    let attestation = parse_attestation_object(raw_attestation)?;
    let cose_key = attestation
        .auth_data
        .attested_data
        .ok_or_else(|| eyre::eyre!("No attested credential data"))?
        .cose_key;

    crate::utils::passkey::passkey_signature_from_public_key::get_public_key_bytes_from_passkey_signature(
        &cose_key,
    )
}

fn parse_attestation_object(
    raw_attestation: &[u8],
) -> Result<AttestationObject> {
    debug!("Raw attestation hex: {}", hex::encode(raw_attestation));

    let value: Value = ciborium::de::from_reader(raw_attestation)
        .map_err(|e| eyre::eyre!("Failed to parse CBOR: {}", e))?;

    debug!("Parsed CBOR value: {:?}", value);

    let map = value
        .as_map()
        .ok_or_else(|| eyre::eyre!("Attestation object is not a CBOR map"))?;

    let auth_data_bytes = map
        .iter()
        .find(|(k, _)| k.as_text() == Some("authData"))
        .and_then(|(_, v)| v.as_bytes())
        .ok_or_else(|| eyre::eyre!("Missing or invalid authData"))?;

    let fmt = map
        .iter()
        .find(|(k, _)| k.as_text() == Some("fmt"))
        .and_then(|(_, v)| v.as_text())
        .ok_or_else(|| eyre::eyre!("Missing or invalid fmt"))?
        .to_string();

    let auth_data = parse_authenticator_data(auth_data_bytes)?;

    // TODO: Parse attestation statement
    let att_stmt = AttestationStatement { x5c: vec![] };

    Ok(AttestationObject { auth_data, fmt, att_stmt })
}

fn parse_authenticator_data(data: &[u8]) -> Result<AuthenticatorData> {
    debug!("Parsing authenticator data: {}", hex::encode(data));

    if data.len() < 37 {
        return Err(eyre::eyre!("Auth data too short"));
    }

    let mut rp_id_hash = [0u8; 32];
    rp_id_hash.copy_from_slice(&data[0..32]);

    let flags = data[32];
    let counter = u32::from_be_bytes(data[33..37].try_into()?);

    debug!("RP ID Hash: {}", hex::encode(rp_id_hash));
    debug!("Flags: {:08b}", flags);
    debug!("Counter: {}", counter);

    let attested_data = if (flags & 0b01000000) != 0 {
        debug!("AT flag set, parsing attested credential data...");
        Some(parse_attested_credential_data(&data[37..])?)
    } else {
        debug!("No AT flag, skipping attested credential data");
        None
    };

    Ok(AuthenticatorData { rp_id_hash, flags, counter, attested_data })
}

fn parse_attested_credential_data(
    data: &[u8],
) -> Result<AttestedCredentialData> {
    if data.len() < 18 {
        return Err(eyre::eyre!("Attested credential data too short"));
    }

    let mut aaguid = [0u8; 16];
    aaguid.copy_from_slice(&data[0..16]);

    let cred_id_len = u16::from_be_bytes(data[16..18].try_into()?) as usize;

    if data.len() < 18 + cred_id_len {
        return Err(eyre::eyre!("Not enough bytes for credential ID"));
    }

    let credential_id = data[18..18 + cred_id_len].to_vec();
    let cose_key = data[18 + cred_id_len..].to_vec();

    Ok(AttestedCredentialData { aaguid, credential_id, cose_key })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ciborium::Value;

    fn create_mock_cose_key() -> Vec<u8> {
        // Example COSE key bytes
        vec![5, 6, 7, 8]
    }

    fn create_mock_attested_data() -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&[0u8; 16]); // AAGUID
        data.extend_from_slice(&[0, 4]); // Credential ID length (4 bytes)
        data.extend_from_slice(&[1, 2, 3, 4]); // Credential ID
        data.extend_from_slice(&create_mock_cose_key()); // COSE key
        data
    }

    fn create_mock_auth_data() -> Vec<u8> {
        let mut auth_data = Vec::new();
        auth_data.extend_from_slice(&[0u8; 32]); // RP ID hash
        auth_data.push(0b01000000); // Flags (AT flag set)
        auth_data.extend_from_slice(&[0, 0, 0, 0]); // Counter
        auth_data.extend_from_slice(&create_mock_attested_data());
        auth_data
    }

    fn create_mock_attestation_object() -> Vec<u8> {
        let map = vec![
            (
                Value::Text("authData".to_string()),
                Value::Bytes(create_mock_auth_data()),
            ),
            (Value::Text("fmt".to_string()), Value::Text("apple".to_string())),
            (
                Value::Text("attStmt".to_string()),
                Value::Map(vec![(
                    Value::Text("x5c".to_string()),
                    Value::Array(vec![Value::Bytes(vec![9, 9, 9])]),
                )]),
            ),
        ];

        let mut raw_attestation = Vec::new();
        ciborium::ser::into_writer(&Value::Map(map), &mut raw_attestation)
            .expect("Failed to serialize CBOR");
        raw_attestation
    }

    #[test]
    fn test_parse_attested_credential_data() -> Result<()> {
        let data = create_mock_attested_data();
        let result = parse_attested_credential_data(&data)?;

        assert_eq!(result.aaguid, [0u8; 16]);
        assert_eq!(result.credential_id, vec![1, 2, 3, 4]);
        assert_eq!(result.cose_key, create_mock_cose_key());
        Ok(())
    }

    #[test]
    fn test_parse_authenticator_data() -> Result<()> {
        let auth_data = create_mock_auth_data();
        let result = parse_authenticator_data(&auth_data)?;

        assert_eq!(result.rp_id_hash, [0u8; 32]);
        assert_eq!(result.flags, 0b01000000);
        assert_eq!(result.counter, 0);

        let attested_data =
            result.attested_data.expect("Should have attested data");
        assert_eq!(attested_data.cose_key, create_mock_cose_key());
        Ok(())
    }

    #[test]
    fn test_parse_attestation_object() -> Result<()> {
        let raw_attestation = create_mock_attestation_object();
        let result = parse_attestation_object(&raw_attestation)?;

        assert_eq!(result.fmt, "apple");
        assert_eq!(result.auth_data.flags, 0b01000000);
        assert!(result.auth_data.attested_data.is_some());
        Ok(())
    }

    #[test]
    fn test_extract_public_key() -> Result<()> {
        let raw_attestation = create_mock_attestation_object();
        let result = extract_public_key(&raw_attestation);

        // This will fail until we implement proper COSE key parsing
        // Just testing that the extraction chain works
        assert!(result.is_err());
        Ok(())
    }
}
