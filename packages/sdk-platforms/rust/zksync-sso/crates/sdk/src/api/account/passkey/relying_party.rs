#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidRpId {
    pub origin: String,
    pub rp_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpId {
    Apple(String),
    Android(AndroidRpId),
}

impl RpId {
    pub fn origin(&self) -> String {
        match self {
            RpId::Apple(rp_id) => format!("https://{rp_id}"),
            RpId::Android(android_rp_id) => android_rp_id.origin.to_string(),
        }
    }

    pub fn rp_id(&self) -> String {
        match self {
            RpId::Apple(rp_id) => rp_id.to_string(),
            RpId::Android(android_rp_id) => android_rp_id.rp_id.to_string(),
        }
    }
}
