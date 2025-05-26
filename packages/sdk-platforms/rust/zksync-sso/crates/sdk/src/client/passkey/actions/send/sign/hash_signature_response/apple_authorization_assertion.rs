use base64_url::base64::{Engine, engine::general_purpose::STANDARD};
use eyre::Result;
use serde::{Deserialize, Serialize, de::Deserializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthorizationPublicKeyCredentialAttachment {
    #[serde(rename = "platform")]
    Platform,
    #[serde(rename = "crossPlatform")]
    CrossPlatform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationPlatformPublicKeyCredentialAssertion {
    pub attachment: AuthorizationPublicKeyCredentialAttachment,
    #[serde(deserialize_with = "deserialize_base64")]
    pub raw_authenticator_data: Vec<u8>,
    #[serde(rename = "userID")]
    #[serde(deserialize_with = "deserialize_base64")]
    pub user_id: Vec<u8>,
    #[serde(deserialize_with = "deserialize_base64")]
    pub signature: Vec<u8>,
    #[serde(rename = "credentialID")]
    #[serde(deserialize_with = "deserialize_base64")]
    pub credential_id: Vec<u8>,
    #[serde(rename = "rawClientDataJSON")]
    #[serde(deserialize_with = "deserialize_base64")]
    pub raw_client_data_json: Vec<u8>,
    pub large_blob:
        Option<AuthorizationPublicKeyCredentialLargeBlobAssertionOutput>,
    pub prf: Option<AuthorizationPublicKeyCredentialPRFAssertionOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationPublicKeyCredentialLargeBlobAssertionOutput {
    pub result: LargeBlobOperationResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum LargeBlobOperationResult {
    #[serde(rename = "read")]
    Read(Option<Vec<u8>>),
    #[serde(rename = "write")]
    Write(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymmetricKey(Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationPublicKeyCredentialPRFAssertionOutput {
    pub first: SymmetricKey,
    pub second: Option<SymmetricKey>,
}

fn deserialize_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer).and_then(|string| {
        STANDARD.decode(&string).map_err(|e| Error::custom(e.to_string()))
    })
}
