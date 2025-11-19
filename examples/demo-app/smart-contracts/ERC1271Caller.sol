// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/utils/cryptography/EIP712.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/interfaces/IERC1271.sol";
import "@openzeppelin/contracts/utils/Address.sol";

contract ERC1271Caller is EIP712 {
  struct TestStruct {
    string message;
    uint256 value;
  }

  constructor() EIP712("ERC1271Caller", "1.0.0") {}

  function validateStruct(
    TestStruct calldata testStruct,
    address signer,
    bytes calldata signature
  ) external view returns (bool) {
    require(signer != address(0), "Invalid signer address");

    bytes32 structHash = keccak256(
      abi.encode(
        keccak256("TestStruct(string message,uint256 value)"),
        keccak256(bytes(testStruct.message)),
        testStruct.value
      )
    );

    bytes32 digest = _hashTypedDataV4(structHash);

    bytes4 magic = IERC1271(signer).isValidSignature(digest, signature);
    return magic == IERC1271.isValidSignature.selector;
  }

  /**
   * Validates a precomputed digest against an ERC-1271 signer.
   * This is useful for schemes like ERC-7739 where the digest is computed off-chain.
   */
  function validateDigest(bytes32 digest, address signer, bytes calldata signature) external view returns (bool) {
    require(signer != address(0), "Invalid signer address");
    bytes4 magic = IERC1271(signer).isValidSignature(digest, signature);
    return magic == IERC1271.isValidSignature.selector;
  }

  // --- DEBUG helpers ---------------------------------------------------
  /// @notice Compute the EIP-712 struct hash for the provided TestStruct
  function computeStructHash(TestStruct calldata testStruct) external pure returns (bytes32) {
    return
      keccak256(
        abi.encode(
          keccak256("TestStruct(string message,uint256 value)"),
          keccak256(bytes(testStruct.message)),
          testStruct.value
        )
      );
  }

  /// @notice Compute the TypedDataSign typed-data wrapper typehash, wrapper struct hash and final hash from Basic.t.sol
  /// @dev This mirrors the assembly algorithm in Solady/Basic.t.sol for debugging parity
  function computeTypedDataSignAndFinalHash(
    TestStruct calldata testStruct,
    string calldata contentsDescription,
    string calldata verifierName,
    string calldata verifierVersion,
    uint256 verifierChainId,
    address verifierContract,
    bytes32 verifierSalt
  ) external view returns (bytes32 typedDataSignTypehash, bytes32 wrapperStructHash, bytes32 finalHash) {
    bytes32 structHash = keccak256(
      abi.encode(
        keccak256("TestStruct(string message,uint256 value)"),
        keccak256(bytes(testStruct.message)),
        testStruct.value
      )
    );

    // Build the TypedDataSign type string, appending the `contentsDescription` as in Basic.t.sol
    bytes memory prefix = abi.encodePacked(
      "TypedDataSign(",
      "TestStruct contents,",
      "string name,",
      "string version,",
      "uint256 chainId,",
      "address verifyingContract,",
      "bytes32 salt)",
      contentsDescription
    );

    typedDataSignTypehash = keccak256(prefix);

    wrapperStructHash = keccak256(
      abi.encode(
        typedDataSignTypehash,
        structHash,
        keccak256(bytes(verifierName)),
        keccak256(bytes(verifierVersion)),
        verifierChainId,
        uint256(uint160(verifierContract)),
        verifierSalt
      )
    );

    finalHash = keccak256(abi.encodePacked(hex"1901", _domainSeparatorV4(), wrapperStructHash));
  }

  /// @notice Return this contract's EIP-712 domain separator (app domain)
  function appDomainSeparator() external view returns (bytes32) {
    return _domainSeparatorV4();
  }
}
