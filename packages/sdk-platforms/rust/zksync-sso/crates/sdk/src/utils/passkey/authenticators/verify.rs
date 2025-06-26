use super::{
    android::verify::verify_registration as android_verify_registration,
    apple::verify::verify_registration as apple_verify_registration,
};
use crate::api::account::deployment::RpId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedPasskey {
    pub public_key: Vec<u8>,
    pub credential_id: Vec<u8>,
    pub cose_key_cbor: Vec<u8>,
}

pub async fn verify_registration(
    raw_attestation_object: &[u8],
    raw_client_data_json: &[u8],
    credential_id: &[u8],
    expected_rp_id: &RpId,
) -> eyre::Result<ValidatedPasskey> {
    match expected_rp_id {
        RpId::Apple(expected_origin) => {
            apple_verify_registration(
                raw_attestation_object,
                raw_client_data_json,
                credential_id,
                expected_origin,
            )
            .await
        }
        RpId::Android(expected_android_rp_id) => {
            android_verify_registration(
                raw_attestation_object,
                raw_client_data_json,
                credential_id,
                &expected_android_rp_id.origin,
            )
            .await
        }
    }
}
