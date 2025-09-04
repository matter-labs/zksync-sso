use crate::account::passkey::relying_party::RpId;
use sdk::api::account::passkey::passkey_parameters::{
    ParsedPasskeyParameters as SdkParsedPasskeyParameters,
    ParsedPasskeyParametersCredential as SdkParsedPasskeyParametersCredential,
    PasskeyParameters as SdkPasskeyParameters,
    parse_passkey_parameters as sdk_parse_passkey_parameters,
};

#[derive(Debug, uniffi::Record)]
pub struct PasskeyParameters {
    pub credential_raw_attestation_object: Vec<u8>,
    pub credential_raw_client_data_json: Vec<u8>,
    pub credential_id: Vec<u8>,
    pub rp_id: RpId,
}

impl From<PasskeyParameters> for SdkPasskeyParameters {
    fn from(passkey_parameters: PasskeyParameters) -> Self {
        SdkPasskeyParameters {
            credential_raw_attestation_object: passkey_parameters
                .credential_raw_attestation_object,
            credential_raw_client_data_json: passkey_parameters
                .credential_raw_client_data_json,
            credential_id: passkey_parameters.credential_id,
            rp_id: passkey_parameters.rp_id.into(),
        }
    }
}

#[derive(Debug, uniffi::Record)]
pub struct ParsedPasskeyParametersCredential {
    pub id: String,
    pub public_key: Vec<u8>,
}

impl From<SdkParsedPasskeyParametersCredential>
    for ParsedPasskeyParametersCredential
{
    fn from(credential: SdkParsedPasskeyParametersCredential) -> Self {
        Self { id: credential.id, public_key: credential.public_key }
    }
}

#[derive(Debug, uniffi::Record)]
pub struct ParsedPasskeyParameters {
    pub credential: ParsedPasskeyParametersCredential,
    pub expected_origin: String,
}

impl From<SdkParsedPasskeyParameters> for ParsedPasskeyParameters {
    fn from(parsed_params: SdkParsedPasskeyParameters) -> Self {
        Self {
            credential: parsed_params.credential.into(),
            expected_origin: parsed_params.expected_origin,
        }
    }
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ParsePasskeyParametersError {
    #[error("{0}")]
    Parse(String),
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn parse_passkey_parameters(
    params: PasskeyParameters,
) -> Result<ParsedPasskeyParameters, ParsePasskeyParametersError> {
    let sdk_params = params.into();

    sdk_parse_passkey_parameters(&sdk_params)
        .await
        .map_err(|e| ParsePasskeyParametersError::Parse(e.to_string()))
        .map(Into::into)
}
