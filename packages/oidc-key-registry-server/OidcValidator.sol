contract OidcValidator is IModuleValidator, VerifierCaller {
  struct OidcData {
    bytes oidcDigest; // PoseidonHash(sub || aud || iss || salt)
    bytes iss; // Issuer
    bytes aud; // Audience
  }

  struct OidcSignature {
    bytes zkProof;
    bytes pkop; // Public key for OP
  }

  mapping(address => OidcData) public accountData;

  /// Associates a new `oidcDigest` for the user, calculated as `PoseidonHash(sub || aud || iss || salt)`.
  /// This digest is used as input for zk proof verification.
  function addValidationKey(bytes calldata key) external returns (bool);

  /// Validates the transaction to add a new passkey for the user.
  /// The signature is interpreted as an `OidcSignature`.
  /// The contract queries `OidcKeyRegistry` for the provider’s public key (`pkop`).
  /// The contract verifies the zk proof calling the verifier contract.
  /// If the proof is valid, it approves the transaction and the `PasskeyModule` adds the new passkey.
  function validateTransaction(
    bytes32 signedHash,
    bytes calldata signature,
    Transaction calldata transaction
  ) external view returns (bool);

  /// Unimplemented signature validation.
  function validateSignature(bytes32 signedHash, bytes memory signature) external view returns (bool);
}
