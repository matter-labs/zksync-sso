use super::PasskeyParameters;
use crate::utils::passkey::authenticators::{
    apple::extract_public_key, verify::verify_registration,
};
use log::debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPasskeyParametersCredential {
    pub id: String,
    pub public_key: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPasskeyParameters {
    pub credential: ParsedPasskeyParametersCredential,
    pub expected_origin: String,
}

pub(crate) async fn parse_passkey_parameters(
    params: &PasskeyParameters,
) -> eyre::Result<ParsedPasskeyParameters> {
    let (old_public_key_x, old_public_key_y) =
        extract_public_key(&params.credential_raw_attestation_object)
            .map_err(|e| {
                eyre::eyre!(
                    "XDB deploy_account - Old method - Failed to parse raw attestation object: {}",
                    e
                )
            })?;

    debug!(
        "XDB deploy_account - Old method - Passkey public key x: {old_public_key_x:?}"
    );
    debug!(
        "XDB deploy_account - Old method - Passkey public key y: {old_public_key_y:?}"
    );

    let validated = verify_registration(
        &params.credential_raw_attestation_object,
        &params.credential_raw_client_data_json,
        &params.credential_id,
        &params.rp_id,
    )
    .await?;

    let public_key = validated.public_key;
    debug!(
        "XDB deploy_account - New method - Passkey public key: {public_key:?}"
    );

    let (public_key_x, public_key_y) = {
        let key_bytes = &public_key[public_key.len() - 65..];
        if key_bytes[0] != 0x04 {
            return Err(eyre::eyre!(
                "XDB deploy_account - Invalid public key format from validation"
            ));
        }
        let x_bytes: [u8; 32] = key_bytes[1..33].try_into().unwrap();
        let y_bytes: [u8; 32] = key_bytes[33..65].try_into().unwrap();
        (x_bytes, y_bytes)
    };
    debug!(
        "XDB deploy_account - New method - Passkey public key x: {public_key_x:?}"
    );
    debug!(
        "XDB deploy_account - New method - Passkey public key y: {public_key_y:?}"
    );

    debug!(
        "XDB deploy_account - Public keys x match: {}",
        old_public_key_x == public_key_x
    );
    debug!(
        "XDB deploy_account - Public keys y match: {}",
        old_public_key_y == public_key_y
    );

    let expected_origin = params.rp_id.origin();
    debug!("XDB deploy_account - Expected origin: {expected_origin}");

    use base64::Engine;
    let id_base64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(validated.credential_id);

    Ok(ParsedPasskeyParameters {
        credential: ParsedPasskeyParametersCredential {
            id: id_base64,
            public_key: validated.cose_key_cbor,
        },
        expected_origin,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::account::deployment::{AndroidRpId, RpId};
    use base64::Engine;
    use eyre::Ok;

    #[tokio::test]
    async fn test_parse_attested_android_credential_data() -> eyre::Result<()> {
        let expected_parsed = ParsedPasskeyParameters {
            credential: ParsedPasskeyParametersCredential {
                id: "AT8liaFjldJ8klbhXX_FOy3-XILmkl226AMjcQVMgTSqPxNLKYqS3nH7NE4pe7AgPgHVsuRUpHcXunesVsNohx4".to_string(),
                public_key: vec![
                    165, 1, 2, 3, 38, 32, 1, 33, 88, 32, 160, 214, 110, 174, 8,
                    63, 179, 70, 206, 183, 25, 125, 102, 92, 245, 10, 215, 210,
                    72, 218, 109, 169, 123, 242, 215, 151, 157, 49, 255, 100,
                    31, 58, 34, 88, 32, 149, 109, 194, 233, 186, 233, 80, 175,
                    222, 255, 222, 58, 119, 11, 79, 27, 152, 227, 30, 15, 250,
                    63, 82, 114, 116, 173, 106, 23, 36, 226, 28, 74,
                ],
            },
            expected_origin: "android:apk-key-hash:-sYXRdwJA3hvue3mKpYrOZ9zSPC7b4mbgzJmdZEDO5w"
                .to_string(),
        };

        let credential_raw_client_data_json_base64 = "eyJ0eXBlIjoid2ViYXV0aG4uY3JlYXRlIiwiY2hhbGxlbmdlIjoiVTVZTE0zZExhSVZxdlc3czdjX3hndEtuNlNOeUlkb0FIOFV3SHBULTM4WSIsIm9yaWdpbiI6ImFuZHJvaWQ6YXBrLWtleS1oYXNoOi1zWVhSZHdKQTNodnVlM21LcFlyT1o5elNQQzdiNG1iZ3pKbWRaRURPNXciLCJhbmRyb2lkUGFja2FnZU5hbWUiOiJ6a3N5bmNzc28uZXhhbXBsZSJ9".to_string();
        let credential_raw_client_data_json =
            base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(credential_raw_client_data_json_base64)?;

        let credential_raw_attestation_object_base64 = "o2NmbXRkbm9uZWdhdHRTdG10oGhhdXRoRGF0YVjF08tFjuLOhgB6vt7kcKBTmkNjX9Yu4Wdm0LLy_MUj0v1FAAAAAAAAAAAAAAAAAAAAAAAAAAAAQQE_JYmhY5XSfJJW4V1_xTst_lyC5pJdtugDI3EFTIE0qj8TSymKkt5x-zROKXuwID4B1bLkVKR3F7p3rFbDaIcepQECAyYgASFYIKDWbq4IP7NGzrcZfWZc9QrX0kjabal78teXnTH_ZB86IlgglW3C6brpUK_e_946dwtPG5jjHg_6P1JydK1qFyTiHEo".to_string();
        let credential_raw_attestation_object =
            base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(credential_raw_attestation_object_base64)?;

        let credential_id_base64 = "AT8liaFjldJ8klbhXX_FOy3-XILmkl226AMjcQVMgTSqPxNLKYqS3nH7NE4pe7AgPgHVsuRUpHcXunesVsNohx4".to_string();
        let credential_id = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(credential_id_base64)?;

        let android_rp_id = AndroidRpId {
            origin: "android:apk-key-hash:-sYXRdwJA3hvue3mKpYrOZ9zSPC7b4mbgzJmdZEDO5w".to_string(),
            rp_id: "soo-sdk-example-pages.pages.dev".to_string(),
        };

        let rp_id = RpId::Android(android_rp_id);

        let params = PasskeyParameters {
            credential_raw_attestation_object,
            credential_raw_client_data_json,
            credential_id,
            rp_id,
        };

        let parsed = parse_passkey_parameters(&params).await?;

        eyre::ensure!(
            parsed == expected_parsed,
            "parsed passkey parameters {:?} do not match expected {:?}",
            parsed,
            expected_parsed
        );

        Ok(())
    }
}
