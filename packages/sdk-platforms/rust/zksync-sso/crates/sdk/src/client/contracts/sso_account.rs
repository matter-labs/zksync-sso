use alloy::sol;

sol! {
    #[derive(Debug, Default)]
    #[sol(rpc)]
    contract SsoAccount {
        function supportsInterface(bytes4 interfaceId) external view returns (bool);

        function validateTransaction(
            bytes32,
            bytes32 suggestedSignedHash,
            Transaction transaction
        ) external payable returns (bytes4);

        function executeTransaction(
            bytes32 suggestedSignedHash,
            Transaction transaction,
            bytes signature
        ) external payable;

        struct Transaction {
            uint256 txType;
            uint256 from;
            uint256 to;
            uint256 gasLimit;
            uint256 gasPerPubdataByteLimit;
            uint256 maxFeePerGas;
            uint256 maxPriorityFeePerGas;
            uint256 paymaster;
            uint256 nonce;
            uint256 value;
            uint256[4] reserved;
            bytes data;
            bytes signature;
            bytes32[] factoryDeps;
            bytes paymasterInput;
            bytes reservedDynamic;
        }
    }
}
