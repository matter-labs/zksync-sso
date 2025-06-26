use crate::utils::passkey::authenticators::verify::ValidatedPasskey;
use log::{debug, error};
use passkey::client::android_extra::{
    AndroidPasskeyRegistration, AndroidValidatedPasskey as PasskeyValidated,
};

impl From<PasskeyValidated> for ValidatedPasskey {
    fn from(validated: PasskeyValidated) -> Self {
        Self {
            public_key: validated.public_key,
            credential_id: validated.credential_id,
            cose_key_cbor: validated.cose_key_cbor,
        }
    }
}

pub async fn verify_registration(
    raw_attestation_object: &[u8],
    raw_client_data_json: &[u8],
    credential_id: &[u8],
    expected_origin: &str,
) -> eyre::Result<ValidatedPasskey> {
    debug!(
        "android::verify_registration - raw_attestation_object: {:?}",
        raw_attestation_object
    );
    debug!(
        "android::verify_registration - raw_client_data_json: {:?}",
        raw_client_data_json
    );
    debug!("android::verify_registration - credential_id: {:?}", credential_id);
    debug!(
        "android::verify_registration - expected_origin: {}",
        expected_origin
    );

    let registration = AndroidPasskeyRegistration {
        raw_attestation_object: raw_attestation_object.to_vec(),
        raw_client_data_json: raw_client_data_json.to_vec(),
        credential_id: credential_id.to_vec(),
    };
    debug!("android::verify_registration - registration: {:?}", registration);

    let client_data: serde_json::Value =
        serde_json::from_slice(raw_client_data_json)?;
    debug!("android::verify_registration - client_data: {:?}", client_data);

    let challenge = base64_url::decode(
        client_data["challenge"]
            .as_str()
            .ok_or_else(|| eyre::eyre!("Missing challenge"))?,
    )?;
    debug!("android::verify_registration - challenge: {:?}", challenge);

    let validated = registration
        .validate(&challenge, expected_origin)
        .map_err(|e| {
            error!("android::verify_registration - Validation failed: {:?}", e);
            eyre::eyre!("Passkey validation failed: {:?}", e)
        })
        .map(Into::into)?;
    debug!("android::verify_registration - validated: {:?}", validated);

    Ok(validated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eyre::Ok;

    #[tokio::test]
    async fn test_verify_registration() -> eyre::Result<()> {
        let expecte_public_key: Vec<u8> = vec![
            48, 89, 48, 19, 6, 7, 42, 134, 72, 206, 61, 2, 1, 6, 8, 42, 134,
            72, 206, 61, 3, 1, 7, 3, 66, 0, 4, 160, 214, 110, 174, 8, 63, 179,
            70, 206, 183, 25, 125, 102, 92, 245, 10, 215, 210, 72, 218, 109,
            169, 123, 242, 215, 151, 157, 49, 255, 100, 31, 58, 149, 109, 194,
            233, 186, 233, 80, 175, 222, 255, 222, 58, 119, 11, 79, 27, 152,
            227, 30, 15, 250, 63, 82, 114, 116, 173, 106, 23, 36, 226, 28, 74,
        ];
        let expecte_credential_id: Vec<u8> = vec![
            1, 63, 37, 137, 161, 99, 149, 210, 124, 146, 86, 225, 93, 127, 197,
            59, 45, 254, 92, 130, 230, 146, 93, 182, 232, 3, 35, 113, 5, 76,
            129, 52, 170, 63, 19, 75, 41, 138, 146, 222, 113, 251, 52, 78, 41,
            123, 176, 32, 62, 1, 213, 178, 228, 84, 164, 119, 23, 186, 119,
            172, 86, 195, 104, 135, 30,
        ];
        let expecte_cose_key_cbor: Vec<u8> = vec![
            165, 1, 2, 3, 38, 32, 1, 33, 88, 32, 160, 214, 110, 174, 8, 63,
            179, 70, 206, 183, 25, 125, 102, 92, 245, 10, 215, 210, 72, 218,
            109, 169, 123, 242, 215, 151, 157, 49, 255, 100, 31, 58, 34, 88,
            32, 149, 109, 194, 233, 186, 233, 80, 175, 222, 255, 222, 58, 119,
            11, 79, 27, 152, 227, 30, 15, 250, 63, 82, 114, 116, 173, 106, 23,
            36, 226, 28, 74,
        ];

        let expected_validated_passkey: ValidatedPasskey = ValidatedPasskey {
            public_key: expecte_public_key,
            credential_id: expecte_credential_id,
            cose_key_cbor: expecte_cose_key_cbor,
        };

        let raw_attestation_object: Vec<u8> = vec![
            163, 99, 102, 109, 116, 100, 110, 111, 110, 101, 103, 97, 116, 116,
            83, 116, 109, 116, 160, 104, 97, 117, 116, 104, 68, 97, 116, 97,
            88, 197, 211, 203, 69, 142, 226, 206, 134, 0, 122, 190, 222, 228,
            112, 160, 83, 154, 67, 99, 95, 214, 46, 225, 103, 102, 208, 178,
            242, 252, 197, 35, 210, 253, 69, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 65, 1, 63, 37, 137, 161, 99, 149,
            210, 124, 146, 86, 225, 93, 127, 197, 59, 45, 254, 92, 130, 230,
            146, 93, 182, 232, 3, 35, 113, 5, 76, 129, 52, 170, 63, 19, 75, 41,
            138, 146, 222, 113, 251, 52, 78, 41, 123, 176, 32, 62, 1, 213, 178,
            228, 84, 164, 119, 23, 186, 119, 172, 86, 195, 104, 135, 30, 165,
            1, 2, 3, 38, 32, 1, 33, 88, 32, 160, 214, 110, 174, 8, 63, 179, 70,
            206, 183, 25, 125, 102, 92, 245, 10, 215, 210, 72, 218, 109, 169,
            123, 242, 215, 151, 157, 49, 255, 100, 31, 58, 34, 88, 32, 149,
            109, 194, 233, 186, 233, 80, 175, 222, 255, 222, 58, 119, 11, 79,
            27, 152, 227, 30, 15, 250, 63, 82, 114, 116, 173, 106, 23, 36, 226,
            28, 74,
        ];

        let raw_client_data_json: Vec<u8> = vec![
            123, 34, 116, 121, 112, 101, 34, 58, 34, 119, 101, 98, 97, 117,
            116, 104, 110, 46, 99, 114, 101, 97, 116, 101, 34, 44, 34, 99, 104,
            97, 108, 108, 101, 110, 103, 101, 34, 58, 34, 85, 53, 89, 76, 77,
            51, 100, 76, 97, 73, 86, 113, 118, 87, 55, 115, 55, 99, 95, 120,
            103, 116, 75, 110, 54, 83, 78, 121, 73, 100, 111, 65, 72, 56, 85,
            119, 72, 112, 84, 45, 51, 56, 89, 34, 44, 34, 111, 114, 105, 103,
            105, 110, 34, 58, 34, 97, 110, 100, 114, 111, 105, 100, 58, 97,
            112, 107, 45, 107, 101, 121, 45, 104, 97, 115, 104, 58, 45, 115,
            89, 88, 82, 100, 119, 74, 65, 51, 104, 118, 117, 101, 51, 109, 75,
            112, 89, 114, 79, 90, 57, 122, 83, 80, 67, 55, 98, 52, 109, 98,
            103, 122, 74, 109, 100, 90, 69, 68, 79, 53, 119, 34, 44, 34, 97,
            110, 100, 114, 111, 105, 100, 80, 97, 99, 107, 97, 103, 101, 78,
            97, 109, 101, 34, 58, 34, 122, 107, 115, 121, 110, 99, 115, 115,
            111, 46, 101, 120, 97, 109, 112, 108, 101, 34, 125,
        ];

        let credential_id: Vec<u8> = vec![
            1, 63, 37, 137, 161, 99, 149, 210, 124, 146, 86, 225, 93, 127, 197,
            59, 45, 254, 92, 130, 230, 146, 93, 182, 232, 3, 35, 113, 5, 76,
            129, 52, 170, 63, 19, 75, 41, 138, 146, 222, 113, 251, 52, 78, 41,
            123, 176, 32, 62, 1, 213, 178, 228, 84, 164, 119, 23, 186, 119,
            172, 86, 195, 104, 135, 30,
        ];

        let expected_origin =
            "android:apk-key-hash:-sYXRdwJA3hvue3mKpYrOZ9zSPC7b4mbgzJmdZEDO5w"
                .to_string();

        let validated_passkey = verify_registration(
            &raw_attestation_object,
            &raw_client_data_json,
            &credential_id,
            &expected_origin,
        )
        .await?;

        eyre::ensure!(
            validated_passkey == expected_validated_passkey,
            "Validated passkey is not as expected"
        );

        Ok(())
    }
}
