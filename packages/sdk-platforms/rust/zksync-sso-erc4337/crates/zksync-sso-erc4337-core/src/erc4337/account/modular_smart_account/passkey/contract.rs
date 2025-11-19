use alloy::sol;

sol!(
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    WebAuthnValidator,
    "../../../../../../packages/erc4337-contracts/out/WebAuthnValidator.sol/WebAuthnValidator.json"
);
