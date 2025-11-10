use alloy::sol;
use std::fmt::Debug;

sol!(
    #[sol(rpc)]
    EntryPoint,
    "../../../../../../packages/erc4337-contracts/out/EntryPoint.sol/EntryPoint.json"
);

pub use self::EntryPoint::PackedUserOperation;

impl Debug for PackedUserOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackedUserOperation")
            .field("sender", &self.sender)
            .field("nonce", &self.nonce)
            .field("initCode", &self.initCode)
            .field("callData", &self.callData)
            .field("accountGasLimits", &self.accountGasLimits)
            .field("preVerificationGas", &self.preVerificationGas)
            .field("gasFees", &self.gasFees)
            .field("paymasterAndData", &self.paymasterAndData)
            .field("signature", &self.signature)
            .finish()
    }
}

impl PartialEq for PackedUserOperation {
    fn eq(&self, other: &Self) -> bool {
        self.sender == other.sender
            && self.nonce == other.nonce
            && self.initCode == other.initCode
            && self.callData == other.callData
            && self.accountGasLimits == other.accountGasLimits
            && self.preVerificationGas == other.preVerificationGas
            && self.gasFees == other.gasFees
            && self.paymasterAndData == other.paymasterAndData
            && self.signature == other.signature
    }
}

impl Eq for PackedUserOperation {}
