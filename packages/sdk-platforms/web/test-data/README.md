# Test Data Generator

This directory contains utilities for generating test data to validate the
Rust ABI encoding implementation.

## Purpose

The Rust `encode_passkey_signature` function must produce identical output to
ethers.js `AbiCoder.encode()`. This test data generator creates known-good
encodings using ethers.js that the Rust tests can compare against.

## Usage

```bash
# Install dependencies (ethers.js)
cd packages/sdk-platforms/web/test-data
npm install

# Generate test data
npm run generate

# Or manually:
node generate-ethers-encoding.mjs > ethers-test-data.json
```

## Files

- `generate-ethers-encoding.mjs` - Generates test cases with ethers.js encoding
- `ethers-test-data.json` - Generated test data (committed to repo)
- `package.json` - Isolated dependencies for this utility

## Note

This utility requires ethers.js, but the main SDK does **not** depend on
ethers. The test data is pre-generated and committed, so you only need to run
this if:

1. Adding new test cases
2. Verifying the Rust implementation after changes
3. Debugging encoding differences

## Test Cases

1. **minimal_empty** - Empty/zero values (edge case)
2. **typical_webauthn** - Real-world WebAuthn response size
3. **long_credential_id** - Edge case with larger credential ID

## Integration

The Rust test `test_encode_passkey_signature_matches_ethers` loads
`ethers-test-data.json` and compares the output byte-by-byte.
