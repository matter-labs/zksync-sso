#!/usr/bin/env node
/**
 * Generate test data for comparing Rust and ethers.js ABI encoding
 *
 * This script creates ABI-encoded passkey signatures using ethers.js
 * so we can compare them with Rust alloy encoding byte-for-byte
 */

import { AbiCoder } from "ethers";

const abiCoder = AbiCoder.defaultAbiCoder();

// Test case 1: Minimal data (from stub signature pattern)
const testCase1 = {
  name: "minimal_empty",
  authenticatorData: new Uint8Array(0),
  clientDataJSON: "",
  r: new Uint8Array(32), // all zeros
  s: new Uint8Array(32), // all zeros
  credentialId: new Uint8Array(0),
};

// Test case 2: Real-world sized data (typical WebAuthn response)
const testCase2 = {
  name: "typical_webauthn",
  authenticatorData: new Uint8Array([
    0x49, 0x96, 0x0d, 0xe5, 0x88, 0x0e, 0x8c, 0x68, 0x74, 0x34, 0x17, 0x0f,
    0x64, 0x76, 0x60, 0x5b, 0x8f, 0xe4, 0xae, 0xb9, 0xa2, 0x86, 0x32, 0xc7,
    0x99, 0x5c, 0xf3, 0xba, 0x83, 0x1d, 0x97, 0x63, 0x01, 0x00, 0x00, 0x00,
    0x00,
  ]),
  clientDataJSON: "{\"type\":\"webauthn.get\",\"challenge\":\"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\",\"origin\":\"http://localhost:3000\"}",
  r: new Uint8Array([
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
    0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
    0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00,
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
  ]),
  s: new Uint8Array([
    0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01,
    0x00, 0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99,
    0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
    0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12,
  ]),
  credentialId: new Uint8Array([
    0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11,
    0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
  ]),
};

// Test case 3: Edge case - long credential ID
const testCase3 = {
  name: "long_credential_id",
  authenticatorData: new Uint8Array([0x01, 0x02, 0x03]),
  clientDataJSON: "test",
  r: new Uint8Array(32).fill(0xaa),
  s: new Uint8Array(32).fill(0xbb),
  credentialId: new Uint8Array(64).fill(0xcc), // Longer than typical
};

function encodeTestCase(testCase) {
  const encoded = abiCoder.encode(
    ["bytes", "string", "bytes32[2]", "bytes"],
    [
      testCase.authenticatorData,
      testCase.clientDataJSON,
      [testCase.r, testCase.s],
      testCase.credentialId,
    ],
  );

  return {
    name: testCase.name,
    input: {
      authenticatorData: Array.from(testCase.authenticatorData),
      clientDataJSON: testCase.clientDataJSON,
      r: Array.from(testCase.r),
      s: Array.from(testCase.s),
      credentialId: Array.from(testCase.credentialId),
    },
    expected: encoded,
  };
}

// Generate test data
const testData = [
  encodeTestCase(testCase1),
  encodeTestCase(testCase2),
  encodeTestCase(testCase3),
];

// Output as JSON for Rust tests to consume
console.log(JSON.stringify(testData, null, 2));
