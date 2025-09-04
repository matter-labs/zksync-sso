use sdk::api::account::passkey::relying_party::{
    AndroidRpId as SdkAndroidRpId, RpId as SdkRpId,
};

#[derive(Debug, uniffi::Record)]
pub struct AndroidRpId {
    pub origin: String,
    pub rp_id: String,
}

impl From<AndroidRpId> for SdkAndroidRpId {
    fn from(android_rp_id: AndroidRpId) -> Self {
        SdkAndroidRpId {
            origin: android_rp_id.origin,
            rp_id: android_rp_id.rp_id,
        }
    }
}

#[derive(Debug, uniffi::Enum)]
pub enum RpId {
    Apple(String),
    Android(AndroidRpId),
}

impl RpId {
    pub fn new_apple(rp_id: String) -> Self {
        Self::Apple(rp_id)
    }

    pub fn new_android(origin: String, rp_id: String) -> Self {
        Self::Android(AndroidRpId { origin, rp_id })
    }

    pub fn origin(&self) -> &str {
        match self {
            RpId::Apple(rp_id) => rp_id,
            RpId::Android(android_rp_id) => &android_rp_id.origin,
        }
    }

    pub fn identifier(&self) -> &str {
        match self {
            RpId::Apple(rp_id) => rp_id,
            RpId::Android(android_rp_id) => &android_rp_id.rp_id,
        }
    }
}

impl From<RpId> for SdkRpId {
    fn from(rp_id: RpId) -> Self {
        match rp_id {
            RpId::Apple(rp_id) => SdkRpId::Apple(rp_id),
            RpId::Android(android_rp_id) => {
                SdkRpId::Android(android_rp_id.into())
            }
        }
    }
}
