use alloy::sol;

sol! {
    #[derive(Debug)]
    #[sol(rpc)]
    contract WebAuthValidator {

        mapping(string originDomain => mapping(address accountAddress => bytes32)) public lowerKeyHalf;
        mapping(string originDomain => mapping(address accountAddress => bytes32)) public upperKeyHalf;

        function validateSignature(bytes32 signedHash, bytes signature) external view returns (bool);

        function rawVerify(
            bytes32 message,
            bytes32[2] rs,
            bytes32[2] pubKey
        ) external view returns (bool);
    }
}
