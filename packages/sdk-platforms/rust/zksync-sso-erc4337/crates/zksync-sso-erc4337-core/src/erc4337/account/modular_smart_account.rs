pub mod add_passkey;
pub mod deploy;
pub mod nonce;
pub mod send;
pub mod session;
pub mod signers;

#[cfg(test)]
pub mod test_utilities;

use alloy::sol;

sol!(
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    MSAFactory,
    "../../../../../../packages/erc4337-contracts/out/MSAFactory.sol/MSAFactory.json"
);

sol!(
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    ModularSmartAccount,
    "../../../../../../packages/erc4337-contracts/out/ModularSmartAccount.sol/ModularSmartAccount.json"
);

sol!(
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    WebAuthnValidator,
    "../../../../../../packages/erc4337-contracts/out/WebAuthnValidator.sol/WebAuthnValidator.json"
);
