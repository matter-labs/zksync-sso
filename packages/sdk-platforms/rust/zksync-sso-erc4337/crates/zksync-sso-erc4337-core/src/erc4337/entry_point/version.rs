use serde::{Deserialize, Serialize};

pub const ENTRYPOINT_V07_TYPE: &str = "v0.7";
pub const ENTRYPOINT_V08_TYPE: &str = "v0.8";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
pub enum EntryPointVersion {
    V07,
    V08,
}

impl EntryPointVersion {
    pub fn type_string(&self) -> String {
        match self {
            EntryPointVersion::V07 => ENTRYPOINT_V07_TYPE.to_string(),
            EntryPointVersion::V08 => ENTRYPOINT_V08_TYPE.to_string(),
        }
    }

    pub fn is_v07(&self) -> bool {
        self == &EntryPointVersion::V07
    }

    pub fn is_v08(&self) -> bool {
        self == &EntryPointVersion::V08
    }
}

impl From<EntryPointVersion> for String {
    fn from(value: EntryPointVersion) -> Self {
        value.type_string()
    }
}

impl From<String> for EntryPointVersion {
    fn from(value: String) -> Self {
        match value.as_str() {
            ENTRYPOINT_V07_TYPE => EntryPointVersion::V07,
            ENTRYPOINT_V08_TYPE => EntryPointVersion::V08,
            _ => panic!("invalid version string"),
        }
    }
}
