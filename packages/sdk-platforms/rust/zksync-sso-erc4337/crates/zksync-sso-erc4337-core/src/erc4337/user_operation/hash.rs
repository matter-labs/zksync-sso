use core::fmt;

pub mod combine;
pub mod user_operation_hash;
pub mod v07;
pub mod v08;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SentUserOperationHash(String);

impl From<SentUserOperationHash> for String {
    fn from(user_operation_hash: SentUserOperationHash) -> Self {
        user_operation_hash.0
    }
}

impl fmt::Display for SentUserOperationHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
