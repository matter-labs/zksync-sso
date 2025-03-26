use alloy::sol;

sol! {
    #[derive(Debug)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract AAFactory {
        event AccountCreated(address indexed accountAddress, string uniqueAccountId);

        bytes32 public immutable beaconProxyBytecodeHash;
        address public immutable beacon;
        mapping(string => address) public accountMappings;

        constructor(bytes32 _beaconProxyBytecodeHash, address _beacon);

        function getEncodedBeacon() external view returns (bytes memory);

        function deployProxySsoAccount(
            bytes32 _salt,
            string calldata _uniqueAccountId,
            bytes[] calldata _initialValidators,
            address[] calldata _initialK1Owners
        ) external returns (address accountAddress);
    }
}

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

sol! {
    #[derive(Debug)]
    #[sol(rpc)]
    contract WebAuthValidator {
        function validateSignature(bytes32 signedHash, bytes signature) external view returns (bool);
        function rawVerify(
            bytes32 message,
            bytes32[2] rs,
            bytes32[2] pubKey
        ) external view returns (bool);
    }
}
