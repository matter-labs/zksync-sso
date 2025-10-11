pub mod add_passkey;
pub mod deploy;
pub mod nonce;
pub mod send;
pub mod session;
pub mod signature;

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
    IMSA,
    "../../../../../../packages/erc4337-contracts/out/IMSA.sol/IMSA.json"
);

sol!(
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    WebAuthnValidator,
    "../../../../../../packages/erc4337-contracts/out/WebAuthnValidator.sol/WebAuthnValidator.json"
);
