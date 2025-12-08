/**
 * ABI for the GuardianExecutor ERC-4337 module
 * Extracted from packages/erc4337-contracts/out/GuardianExecutor.sol/GuardianExecutor.json
 */
export const GuardianExecutorAbi = [
  {
    type: "constructor",
    inputs: [
      { name: "webAuthValidator", type: "address", internalType: "address" },
      { name: "eoaValidator", type: "address", internalType: "address" },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "EOA_VALIDATOR",
    inputs: [],
    outputs: [{ name: "", type: "address", internalType: "address" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "REQUEST_DELAY_TIME",
    inputs: [],
    outputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "REQUEST_VALIDITY_TIME",
    inputs: [],
    outputs: [{ name: "", type: "uint256", internalType: "uint256" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "WEBAUTHN_VALIDATOR",
    inputs: [],
    outputs: [{ name: "", type: "address", internalType: "address" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "acceptGuardian",
    inputs: [
      { name: "accountToGuard", type: "address", internalType: "address" },
    ],
    outputs: [{ name: "", type: "bool", internalType: "bool" }],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "discardRecovery",
    inputs: [],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "finalizeRecovery",
    inputs: [
      { name: "account", type: "address", internalType: "address" },
      { name: "data", type: "bytes", internalType: "bytes" },
    ],
    outputs: [{ name: "returnData", type: "bytes", internalType: "bytes" }],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "guardianStatusFor",
    inputs: [
      { name: "account", type: "address", internalType: "address" },
      { name: "guardian", type: "address", internalType: "address" },
    ],
    outputs: [
      { name: "isPresent", type: "bool", internalType: "bool" },
      { name: "isActive", type: "bool", internalType: "bool" },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "guardiansFor",
    inputs: [{ name: "account", type: "address", internalType: "address" }],
    outputs: [{ name: "", type: "address[]", internalType: "address[]" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "initializeRecovery",
    inputs: [
      {
        name: "accountToRecover",
        type: "address",
        internalType: "address",
      },
      {
        name: "recoveryType",
        type: "uint8",
        internalType: "enum GuardianExecutor.RecoveryType",
      },
      { name: "data", type: "bytes", internalType: "bytes" },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "isInitialized",
    inputs: [{ name: "account", type: "address", internalType: "address" }],
    outputs: [{ name: "", type: "bool", internalType: "bool" }],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "isModuleType",
    inputs: [{ name: "moduleType", type: "uint256", internalType: "uint256" }],
    outputs: [{ name: "", type: "bool", internalType: "bool" }],
    stateMutability: "pure",
  },
  {
    type: "function",
    name: "onInstall",
    inputs: [{ name: "", type: "bytes", internalType: "bytes" }],
    outputs: [],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "onUninstall",
    inputs: [{ name: "", type: "bytes", internalType: "bytes" }],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "pendingRecovery",
    inputs: [{ name: "account", type: "address", internalType: "address" }],
    outputs: [
      {
        name: "recoveryType",
        type: "uint8",
        internalType: "enum GuardianExecutor.RecoveryType",
      },
      { name: "hashedData", type: "bytes32", internalType: "bytes32" },
      { name: "timestamp", type: "uint48", internalType: "uint48" },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "proposeGuardian",
    inputs: [
      { name: "newGuardian", type: "address", internalType: "address" },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "removeGuardian",
    inputs: [
      { name: "guardianToRemove", type: "address", internalType: "address" },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "supportsInterface",
    inputs: [{ name: "interfaceId", type: "bytes4", internalType: "bytes4" }],
    outputs: [{ name: "", type: "bool", internalType: "bool" }],
    stateMutability: "pure",
  },
  {
    type: "event",
    name: "GuardianAdded",
    inputs: [
      { name: "account", type: "address", indexed: true, internalType: "address" },
      { name: "guardian", type: "address", indexed: true, internalType: "address" },
    ],
    anonymous: false,
  },
  {
    type: "event",
    name: "GuardianProposed",
    inputs: [
      { name: "account", type: "address", indexed: true, internalType: "address" },
      { name: "guardian", type: "address", indexed: true, internalType: "address" },
    ],
    anonymous: false,
  },
  {
    type: "event",
    name: "GuardianRemoved",
    inputs: [
      { name: "account", type: "address", indexed: true, internalType: "address" },
      { name: "guardian", type: "address", indexed: true, internalType: "address" },
    ],
    anonymous: false,
  },
  {
    type: "event",
    name: "RecoveryDiscarded",
    inputs: [
      { name: "account", type: "address", indexed: true, internalType: "address" },
    ],
    anonymous: false,
  },
  {
    type: "event",
    name: "RecoveryFinished",
    inputs: [
      { name: "account", type: "address", indexed: true, internalType: "address" },
    ],
    anonymous: false,
  },
  {
    type: "event",
    name: "RecoveryInitiated",
    inputs: [
      { name: "account", type: "address", indexed: true, internalType: "address" },
      { name: "guardian", type: "address", indexed: true, internalType: "address" },
      {
        name: "request",
        type: "tuple",
        indexed: false,
        internalType: "struct GuardianExecutor.RecoveryRequest",
        components: [
          {
            name: "recoveryType",
            type: "uint8",
            internalType: "enum GuardianExecutor.RecoveryType",
          },
          { name: "hashedData", type: "bytes32", internalType: "bytes32" },
          { name: "timestamp", type: "uint48", internalType: "uint48" },
        ],
      },
      { name: "recoveryData", type: "bytes", indexed: false, internalType: "bytes" },
    ],
    anonymous: false,
  },
  {
    type: "error",
    name: "AlreadyInitialized",
    inputs: [{ name: "smartAccount", type: "address", internalType: "address" }],
  },
  { type: "error", name: "EmptyRecoveryData", inputs: [] },
  {
    type: "error",
    name: "EnumerableMapNonexistentKey",
    inputs: [{ name: "key", type: "bytes32", internalType: "bytes32" }],
  },
  {
    type: "error",
    name: "GuardianAlreadyPresent",
    inputs: [
      { name: "account", type: "address", internalType: "address" },
      { name: "guardian", type: "address", internalType: "address" },
    ],
  },
  {
    type: "error",
    name: "GuardianInvalidAddress",
    inputs: [{ name: "guardian", type: "address", internalType: "address" }],
  },
  {
    type: "error",
    name: "GuardianNotActive",
    inputs: [
      { name: "account", type: "address", internalType: "address" },
      { name: "guardian", type: "address", internalType: "address" },
    ],
  },
  {
    type: "error",
    name: "GuardianNotFound",
    inputs: [
      { name: "account", type: "address", internalType: "address" },
      { name: "guardian", type: "address", internalType: "address" },
    ],
  },
  {
    type: "error",
    name: "NoRecoveryInProgress",
    inputs: [{ name: "account", type: "address", internalType: "address" }],
  },
  {
    type: "error",
    name: "NotInitialized",
    inputs: [{ name: "smartAccount", type: "address", internalType: "address" }],
  },
  {
    type: "error",
    name: "RecoveryDataMismatch",
    inputs: [
      { name: "savedHash", type: "bytes32", internalType: "bytes32" },
      { name: "providedHash", type: "bytes32", internalType: "bytes32" },
    ],
  },
  {
    type: "error",
    name: "RecoveryInProgress",
    inputs: [{ name: "account", type: "address", internalType: "address" }],
  },
  {
    type: "error",
    name: "RecoveryTimestampInvalid",
    inputs: [{ name: "timestamp", type: "uint48", internalType: "uint48" }],
  },
  {
    type: "error",
    name: "UnsupportedRecoveryType",
    inputs: [
      {
        name: "recoveryType",
        type: "uint8",
        internalType: "enum GuardianExecutor.RecoveryType",
      },
    ],
  },
  {
    type: "error",
    name: "ValidatorNotInstalled",
    inputs: [
      { name: "account", type: "address", internalType: "address" },
      { name: "validator", type: "address", internalType: "address" },
    ],
  },
] as const;

/**
 * RecoveryType enum values matching the Solidity contract
 */
export enum RecoveryType {
  None = 0,
  WebAuthn = 1,
  EOA = 2,
}
