# EIP-712 + ERC-7739 Typed Data Demo (Web SDK)

This document outlines how we add a concrete example of EIP-712 signing wrapped
with ERC-7739 and validate it on-chain via ERC-1271 in the demo app.

## Goals

- Demonstrate how a smart account (non-EOA) signs typed data following ERC-7739
  (nested EIP-712) and validates it via ERC-1271.
- Provide a reference for NFT marketplaces and dapps that need typed-data
  signatures from smart accounts.
- Keep CI green (no changes required to Rust; JS/TS additions only).

## Architecture Snapshot

- Account-level validation is implemented by `ERC1271Handler` and validator
  modules (EOA/WebAuthn) in `packages/erc4337-contracts`.
- ERC-7739 support is present via OZ draft utils and is referenced in
  `ERC1271Handler`.
- Integration tests show the pattern for wrapping and validating typed data
  using `viem/experimental/erc7739` and a mock caller.
- The demo contract `examples/demo-app/smart-contracts/ERC1271Caller.sol`
  exposes `validateStruct(TestStruct,address,bytes)` for verification.

## Implementation Plan

1. Add a new section to `examples/demo-app/pages/web-sdk-test.vue` named "Typed
   Data (ERC-7739)".
2. Build typed data (`types`, `primaryType`, `message`) for
   `ERC1271Caller.TestStruct`.
3. Fetch the EIP-712 domain from the deployed `ERC1271Caller` via
   `publicClient.getEip712Domain`, omitting `salt` for compatibility.
4. Sign typed data using the SSO connector + wagmi with ERC-7739 wrapping
   (auth-server path extends with `erc7739Actions`).
5. Display the encoded signature and verify it on-chain with
   `readContract(ERC1271Caller.validateStruct)` using the smart account address
   as `signer`.
6. Show Valid/Invalid result and surface the `ERC1271Caller` address from
   `examples/demo-app/forge-output-erc1271.json`.

## References

- `packages/erc4337-contracts/src/core/ERC1271Handler.sol` (uses ERC-7739,
  forwards to validator via `isValidSignatureWithSender`).
- Tests: `packages/erc4337-contracts/test/integration/basic.test.ts` and
  `passkey.test.ts` for ERC-7739 wrapping and validation.
- Existing demo pattern in `examples/demo-app/pages/index.vue` (typed-data
  section & verification).

## Gaps & Notes

- `packages/sdk-4337` passkey/ecdsa/session accounts do not implement
  `signTypedData`; use the auth-server wallet client path (which is extended
  with `erc7739Actions`).
- Ensure a smart account is connected; `ERC1271Caller` expects
  `IERC1271.isValidSignature` at the `signer` address.
- Domain salt is omitted to match current verification; align on a salt policy
  later.

## Deploy & Try

- This demo currently works only on Anvil (local EVM).

- Option A: Deploy only the `ERC1271Caller` used by the demo (recommended for
  this demo):

```bash
anvil
nx run demo-app:deploy-erc1271-caller
```

- Option B (4337 dev flow on Anvil): Deploy 4337 factory + ERC1271Caller, then
  run dev:

```bash
anvil
nx run demo-app:dev:erc4337:anvil
```

- Start the demo app and open the Web SDK Test page:

```bash
nx run demo-app:dev
```

- In the "Typed Data (ERC-7739)" section, connect, sign, and verify on-chain.

Notes:

- Requires a local Anvil at `http://localhost:8545`.
- Uses an Anvil test private key for deployment (configurable via
  `PRIVATE_KEY`).
