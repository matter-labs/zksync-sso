pub mod deploy;
pub mod nonce;
pub mod send;
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
