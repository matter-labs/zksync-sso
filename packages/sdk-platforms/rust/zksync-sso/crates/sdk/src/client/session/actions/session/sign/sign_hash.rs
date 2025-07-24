use super::session_signer::private_key_to_signer;
use alloy::primitives::{Bytes, FixedBytes};

#[derive(Clone, Debug)]
pub struct CreateTransactionSessionSignedHashParameters {
    pub hash: FixedBytes<32>,
    pub session_key: FixedBytes<32>,
}

pub fn create_transaction_session_signed_hash(
    parameters: CreateTransactionSessionSignedHashParameters,
) -> eyre::Result<Bytes> {
    let session_key_signer = private_key_to_signer(parameters.session_key)?;
    let hash_signature = session_key_signer.sign(parameters.hash)?;
    Ok(hash_signature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{fixed_bytes, hex};

    #[test]
    fn test_signs_hash_with_session_key_detailed_logging() {
        println!(
            "\n=== CREATE SESSION TRANSACTION SIGNATURE HASH SIGNATURE - DETAILED BREAKDOWN ==="
        );

        // Test Vector 1 from TypeScript tests
        let test_hash = fixed_bytes!(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        let session_key = fixed_bytes!(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        );
        let expected_signature = "0x2cbd400991f55c3cde09d24d10d01bf48f7b2a319ef0e6f1a370b65bd4cfc57a4d7e69b218c9499831286c462155c7769345da24dd406d23ad5eaee540cecdea1b";

        println!("Test Vector: Test Vector 1");
        println!("Input parameters:");
        println!("- hash: 0x{}", hex::encode(test_hash));
        println!("- hash (bytes): {:?}", test_hash.as_slice());
        println!(
            "- hash (32 bytes): {}",
            test_hash
                .as_slice()
                .iter()
                .map(|b| format!("0x{b:02x}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
        println!("- sessionKey: 0x{}", hex::encode(session_key));
        println!("- sessionKey (bytes): {:?}", session_key.as_slice());
        println!(
            "- sessionKey (32 bytes): {}",
            session_key
                .as_slice()
                .iter()
                .map(|b| format!("0x{b:02x}"))
                .collect::<Vec<_>>()
                .join(" ")
        );

        let parameters = CreateTransactionSessionSignedHashParameters {
            hash: test_hash,
            session_key,
        };

        let result =
            create_transaction_session_signed_hash(parameters).unwrap();

        println!("\n=== HASH SIGNATURE RESULT ===");
        println!("Signature result: 0x{}", hex::encode(&result));
        println!("Signature (bytes): {:?}", result.as_ref());
        println!(
            "Signature length: {} characters",
            hex::encode(&result).len() + 2
        );
        println!("Signature bytes length: {} bytes", result.len());

        // Break down the signature into r, s, v components (65 bytes total)
        if result.len() == 65 {
            let r = &result[0..32];
            let s = &result[32..64];
            let v = &result[64..65];

            println!("\nSignature components:");
            println!("- r (32 bytes): 0x{}", hex::encode(r));
            println!("- s (32 bytes): 0x{}", hex::encode(s));
            println!("- v (1 byte): 0x{} ({})", hex::encode(v), v[0]);

            println!("\nSignature components (byte arrays):");
            println!(
                "- r: {}",
                r.iter()
                    .map(|b| format!("0x{b:02x}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            println!(
                "- s: {}",
                s.iter()
                    .map(|b| format!("0x{b:02x}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            println!(
                "- v: {}",
                v.iter()
                    .map(|b| format!("0x{b:02x}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }

        // Verify the result structure
        let result_hex = format!("0x{}", hex::encode(&result));
        assert!(result_hex.starts_with("0x"));
        assert_eq!(result_hex.len(), 132); // 65 bytes * 2 + 2 for 0x prefix

        println!("Expected signature: {expected_signature}");
        println!("Signatures match: {}", result_hex == expected_signature);

        // Assert that the signature matches the expected output exactly
        assert_eq!(
            result_hex, expected_signature,
            "Generated signature does not match expected signature from TypeScript tests"
        );
        assert_eq!(result.len(), 65);
    }

    #[test]
    fn test_produces_deterministic_signatures_for_same_input() {
        println!("\n=== DETERMINISTIC SIGNATURE TEST ===");
        println!("Testing with: Test Vector 1");

        let test_hash = fixed_bytes!(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        let session_key = fixed_bytes!(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        );

        let parameters = CreateTransactionSessionSignedHashParameters {
            hash: test_hash,
            session_key,
        };

        let result1 =
            create_transaction_session_signed_hash(parameters.clone()).unwrap();
        let result2 =
            create_transaction_session_signed_hash(parameters.clone()).unwrap();
        let result3 =
            create_transaction_session_signed_hash(parameters).unwrap();

        let result1_hex = format!("0x{}", hex::encode(&result1));
        let result2_hex = format!("0x{}", hex::encode(&result2));
        let result3_hex = format!("0x{}", hex::encode(&result3));

        println!("Signature 1: {result1_hex}");
        println!("Signature 2: {result2_hex}");
        println!("Signature 3: {result3_hex}");
        println!(
            "All signatures identical: {}",
            result1_hex == result2_hex && result2_hex == result3_hex
        );

        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
    }

    #[test]
    fn test_produces_different_signatures_for_different_hashes() {
        println!("\n=== DIFFERENT HASHES SIGNATURE TEST ===");

        let hash1 = fixed_bytes!(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        let hash2 = fixed_bytes!(
            "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"
        );
        let session_key = fixed_bytes!(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        );

        println!("Hash 1: 0x{}", hex::encode(hash1));
        println!("Hash 2: 0x{}", hex::encode(hash2));
        println!("Session Key: 0x{}", hex::encode(session_key));

        let parameters1 = CreateTransactionSessionSignedHashParameters {
            hash: hash1,
            session_key,
        };

        let parameters2 = CreateTransactionSessionSignedHashParameters {
            hash: hash2,
            session_key,
        };

        let signature1 =
            create_transaction_session_signed_hash(parameters1).unwrap();
        let signature2 =
            create_transaction_session_signed_hash(parameters2).unwrap();

        let signature1_hex = format!("0x{}", hex::encode(&signature1));
        let signature2_hex = format!("0x{}", hex::encode(&signature2));

        println!("Signature 1: {signature1_hex}");
        println!("Signature 2: {signature2_hex}");
        println!(
            "Signatures are different: {}",
            signature1_hex != signature2_hex
        );

        assert_ne!(signature1, signature2);
    }

    #[test]
    fn test_produces_different_signatures_for_different_session_keys() {
        println!("\n=== DIFFERENT SESSION KEYS SIGNATURE TEST ===");

        let hash = fixed_bytes!(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        let session_key1 = fixed_bytes!(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        );
        let session_key2 = fixed_bytes!(
            "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
        );

        println!("Hash: 0x{}", hex::encode(hash));
        println!("Session Key 1: 0x{}", hex::encode(session_key1));
        println!("Session Key 2: 0x{}", hex::encode(session_key2));

        let parameters1 = CreateTransactionSessionSignedHashParameters {
            hash,
            session_key: session_key1,
        };

        let parameters2 = CreateTransactionSessionSignedHashParameters {
            hash,
            session_key: session_key2,
        };

        let signature1 =
            create_transaction_session_signed_hash(parameters1).unwrap();
        let signature2 =
            create_transaction_session_signed_hash(parameters2).unwrap();

        let signature1_hex = format!("0x{}", hex::encode(&signature1));
        let signature2_hex = format!("0x{}", hex::encode(&signature2));

        println!("Signature 1: {signature1_hex}");
        println!("Signature 2: {signature2_hex}");
        println!(
            "Signatures are different: {}",
            signature1_hex != signature2_hex
        );

        assert_ne!(signature1, signature2);
    }

    #[test]
    fn test_handles_edge_case_hashes() {
        println!("\n=== EDGE CASE HASHES TEST ===");

        let edge_cases = [
            (
                "All zeros",
                fixed_bytes!(
                    "0x0000000000000000000000000000000000000000000000000000000000000000"
                ),
                "0xb8823364c90ea0d2700d5ad0fe39d16778bc07ce7df4779ff35e4b2660d043cb74a002439225d1d518f9f1cf3db005f5e143196543fd5146a34bf63f0b810ade1b",
            ),
            (
                "All ones",
                fixed_bytes!(
                    "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                ),
                "0x2fa7375140033bff84508b277e7b35831074d863c1ab15cd89b79dc48e98ce0a02bd60523d8cd49a5616a637b061a4a47d5094d8d2b6003d42bf1909efcf93bc1c",
            ),
            (
                "Alternating pattern",
                fixed_bytes!(
                    "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                ),
                "0x0cee6dc8908039deccd0bc9897ce8cfc9ab954315c91d279846ed0bb01d2c1be49d41df6d2d0fa99392a09b98e92d39b29612a35c3eaf44e5d6cd937f8d785b21b",
            ),
        ];

        let session_key = fixed_bytes!(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        );

        for (name, hash, expected_signature) in &edge_cases {
            println!("\nTesting {name}:");
            println!("Hash: 0x{}", hex::encode(hash));

            let parameters = CreateTransactionSessionSignedHashParameters {
                hash: *hash,
                session_key,
            };

            let signature =
                create_transaction_session_signed_hash(parameters).unwrap();
            let signature_hex = format!("0x{}", hex::encode(&signature));

            println!("Generated Signature: {signature_hex}");
            println!("Expected Signature:  {expected_signature}");
            println!("Signature length: {}", signature_hex.len());

            // Structure validation
            assert!(signature_hex.starts_with("0x"));
            assert_eq!(signature_hex.len(), 132); // 65 bytes * 2 + 2 for 0x prefix
            assert_eq!(signature.len(), 65);

            // Assert that the signature matches the expected output exactly
            assert_eq!(
                signature_hex, *expected_signature,
                "Generated signature for {name} does not match expected signature"
            );
            println!("✓ Structure valid and signature matches expected output");
        }
    }

    #[test]
    fn test_validates_signature_structure() {
        println!("\n=== SIGNATURE STRUCTURE VALIDATION TEST ===");

        let test_hash = fixed_bytes!(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        let session_key = fixed_bytes!(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        );

        let parameters = CreateTransactionSessionSignedHashParameters {
            hash: test_hash,
            session_key,
        };

        let signature =
            create_transaction_session_signed_hash(parameters).unwrap();
        let signature_hex = format!("0x{}", hex::encode(&signature));

        println!("Signature: {signature_hex}");

        // Validate structure
        assert!(signature_hex.starts_with("0x"));
        assert_eq!(signature_hex.len(), 132); // 65 bytes * 2 + 2 for 0x prefix
        assert_eq!(signature.len(), 65);

        // Extract components
        let r = &signature[0..32];
        let s = &signature[32..64];
        let v = &signature[64..65];

        println!("Validation results:");
        println!("- Total length: 65 bytes ✓");
        println!("- r length: 32 bytes ✓");
        println!("- s length: 32 bytes ✓");
        println!("- v length: 1 byte ✓");
        println!(
            "- v value: {} (should be 27 or 28 for Ethereum, or 0/1 for some implementations)",
            v[0]
        );

        // Basic validation that r and s are not zero
        let r_is_zero = r.iter().all(|&byte| byte == 0);
        let s_is_zero = s.iter().all(|&byte| byte == 0);

        println!("- r is not zero: {} ✓", !r_is_zero);
        println!("- s is not zero: {} ✓", !s_is_zero);

        assert!(!r_is_zero);
        assert!(!s_is_zero);
    }

    #[test]
    fn test_comprehensive_test_with_all_test_vectors() {
        println!("\n=== COMPREHENSIVE TEST VECTORS ===");

        let test_vectors = [
            (
                "Test Vector 1",
                fixed_bytes!(
                    "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                ),
                fixed_bytes!(
                    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
                ),
                "0x2cbd400991f55c3cde09d24d10d01bf48f7b2a319ef0e6f1a370b65bd4cfc57a4d7e69b218c9499831286c462155c7769345da24dd406d23ad5eaee540cecdea1b",
            ),
            (
                "Test Vector 2",
                fixed_bytes!(
                    "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"
                ),
                fixed_bytes!(
                    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
                ),
                "0x7268479b4a62b1ff5745f1ecf15655683039f6cc5528d2ad280546640dbe5e2c02479e05303facc6862eade4de53273295acd29d54b95cc36d22a1a93f33174d1c",
            ),
            (
                "Test Vector 3",
                fixed_bytes!(
                    "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                ),
                fixed_bytes!(
                    "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
                ),
                "0x25535388aab144f1b6c43493a0daaa4a83c9c62dcd657c8d38387049f6f45faf17452f5f95332a0a8583412994c30d56a602cedcb23f4528814bc3e3f790f89e1b",
            ),
        ];

        for (name, hash, session_key, expected_signature) in &test_vectors {
            println!("\nTesting {name}:");
            println!("Hash: 0x{}", hex::encode(hash));
            println!("Session Key: 0x{}", hex::encode(session_key));

            let parameters = CreateTransactionSessionSignedHashParameters {
                hash: *hash,
                session_key: *session_key,
            };

            let signature =
                create_transaction_session_signed_hash(parameters).unwrap();
            let signature_hex = format!("0x{}", hex::encode(&signature));

            println!("Generated Signature: {signature_hex}");
            println!("Expected Signature:  {expected_signature}");
            println!("Length: {}", signature_hex.len());

            // Structure validation
            assert!(signature_hex.starts_with("0x"));
            assert_eq!(signature_hex.len(), 132);
            assert_eq!(signature.len(), 65);

            // Assert that the signature matches the expected output exactly
            assert_eq!(
                signature_hex, *expected_signature,
                "Generated signature for {name} does not match expected signature from TypeScript tests"
            );
            println!("✓ Structure valid and signature matches expected output");
        }

        println!("\n✅ All test vectors processed successfully");
    }
}
