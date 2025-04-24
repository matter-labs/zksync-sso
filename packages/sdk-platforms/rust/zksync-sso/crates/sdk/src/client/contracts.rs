use alloy::sol;

sol!(
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    WebAuthValidator,
    "../../../../../contracts/artifacts-zk/src/validators/WebAuthValidator.sol/WebAuthValidator.json"
);

sol!(
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    SsoAccount,
    "../../../../../contracts/artifacts-zk/src/SsoAccount.sol/SsoAccount.json"
);

sol!(
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    AAFactory,
    "../../../../../contracts/artifacts-zk/src/AAFactory.sol/AAFactory.json"
);
