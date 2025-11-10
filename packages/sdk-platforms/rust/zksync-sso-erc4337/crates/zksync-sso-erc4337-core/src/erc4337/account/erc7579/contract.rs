use alloy::sol;

pub mod account {
    use super::*;

    sol!(
        #[sol(rpc)]
        #[derive(Debug, Default)]
        #[allow(missing_docs)]
        IERC7579Account,
        "../../../../../../packages/erc4337-contracts/out/IERC7579Account.sol/IERC7579Account.json"
    );
}

pub mod execution {
    use super::*;

    sol!(
        #[sol(rpc)]
        #[derive(Debug, Default)]
        #[allow(missing_docs)]
        IERC7579TypeExporter,
        "../../../../../../packages/erc4337-contracts/out/IERC7579TypeExporter.sol/IERC7579TypeExporter.json"
    );

    pub use self::IERC7579TypeExporter::Execution;
}
