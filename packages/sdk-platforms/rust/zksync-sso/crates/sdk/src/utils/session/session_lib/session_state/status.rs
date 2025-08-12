use crate::contracts::SessionLib::Status as SessionLibStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    NotInitialized,
    Active,
    Closed,
}

impl Status {
    pub fn is_not_initialized(&self) -> bool {
        self == &Status::NotInitialized
    }

    pub fn is_active(&self) -> bool {
        self == &Status::Active
    }

    pub fn is_closed(&self) -> bool {
        self == &Status::Closed
    }
}

impl From<Status> for u8 {
    fn from(value: Status) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for Status {
    type Error = eyre::Report;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == Status::NotInitialized as u8 => {
                Ok(Status::NotInitialized)
            }
            x if x == Status::Active as u8 => Ok(Status::Active),
            x if x == Status::Closed as u8 => Ok(Status::Closed),
            _ => Err(eyre::eyre!("Invalid status value: {}", value)),
        }
    }
}
impl From<SessionLibStatus> for Status {
    fn from(value: SessionLibStatus) -> Self {
        let value: u8 = value.into();
        Status::try_from(value).unwrap()
    }
}
