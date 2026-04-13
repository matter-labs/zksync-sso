use alloy::sol;

sol!(
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    WebAuthnValidator,
    "../../../../../../packages/contracts/out/WebAuthnValidator.sol/WebAuthnValidator.json"
);
