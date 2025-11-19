use crate::erc4337::entry_point::contract::PackedUserOperation;
use alloy::primitives::{Address, Bytes, U256};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct UserOperationPreSponsorship(pub PackedUserOperation);

impl From<PackedUserOperation> for UserOperationPreSponsorship {
    fn from(op: PackedUserOperation) -> Self {
        Self(op)
    }
}

impl Serialize for UserOperationPreSponsorship {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state =
            serializer.serialize_struct("PackedUserOperation", 9)?;
        state.serialize_field("sender", &self.0.sender)?;
        state.serialize_field("nonce", &self.0.nonce)?;
        state.serialize_field("initCode", &self.0.initCode)?;
        state.serialize_field("callData", &self.0.callData)?;
        state.serialize_field("accountGasLimits", &self.0.accountGasLimits)?;
        state.serialize_field(
            "preVerificationGas",
            &self.0.preVerificationGas,
        )?;
        state.serialize_field("gasFees", &self.0.gasFees)?;
        state.serialize_field("paymasterAndData", &self.0.paymasterAndData)?;
        state.serialize_field("signature", &self.0.signature)?;
        state.end()
    }
}

#[derive(
    Default,
    Clone,
    Debug,
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct SponsorshipResult {
    pub call_gas_limit: U256,
    pub verification_gas_limit: U256,
    pub pre_verification_gas: U256,
    pub paymaster: Address,
    pub paymaster_verification_gas_limit: U256,
    pub paymaster_post_op_gas_limit: U256,
    pub paymaster_data: Bytes,
}

#[derive(
    Default,
    Clone,
    Debug,
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct SponsorshipResponse {
    pub pre_verification_gas: U256,
    pub verification_gas_limit: U256,
    pub call_gas_limit: U256,
    pub paymaster: Address,
    pub paymaster_verification_gas_limit: U256,
    pub paymaster_post_op_gas_limit: U256,
    pub paymaster_data: Bytes,
}

impl SponsorshipResponse {
    pub fn mock() -> Self {
        Self {
            call_gas_limit: U256::from(100000),
            verification_gas_limit: U256::from(100000),
            pre_verification_gas: U256::from(50000),
            paymaster: Address::ZERO,
            paymaster_verification_gas_limit: U256::from(50000),
            paymaster_post_op_gas_limit: U256::from(50000),
            paymaster_data: Bytes::default(),
        }
    }
}
