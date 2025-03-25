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
