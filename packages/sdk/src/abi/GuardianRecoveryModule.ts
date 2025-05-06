export const GuardianRecoveryModuleAbi = [
  {
    inputs: [],
    stateMutability: "nonpayable",
    type: "constructor",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "account",
        type: "address",
      },
      {
        internalType: "address",
        name: "guardian",
        type: "address",
      },
    ],
    name: "AccountAlreadyGuardedByGuardian",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "account",
        type: "address",
      },
      {
        internalType: "address",
        name: "guardian",
        type: "address",
      },
    ],
    name: "AccountNotGuardedByAddress",
    type: "error",
  },
  {
    inputs: [],
    name: "CooldownPeriodNotPassed",
    type: "error",
  },
  {
    inputs: [],
    name: "ExpiredRequest",
    type: "error",
  },
  {
    inputs: [],
    name: "GuardianCannotBeSelf",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "guardian",
        type: "address",
      },
    ],
    name: "GuardianNotFound",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "guardian",
        type: "address",
      },
    ],
    name: "GuardianNotProposed",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "uint256",
        name: "chainId",
        type: "uint256",
      },
    ],
    name: "NO_TIMESTAMP_ASSERTER",
    type: "error",
  },
  {
    inputs: [],
    name: "PasskeyNotMatched",
    type: "error",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "address",
        name: "account",
        type: "address",
      },
      {
        indexed: true,
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
      {
        indexed: true,
        internalType: "address",
        name: "guardian",
        type: "address",
      },
    ],
    name: "GuardianAdded",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "address",
        name: "account",
        type: "address",
      },
      {
        indexed: true,
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
      {
        indexed: true,
        internalType: "address",
        name: "guardian",
        type: "address",
      },
    ],
    name: "GuardianProposed",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "address",
        name: "account",
        type: "address",
      },
      {
        indexed: true,
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
      {
        indexed: true,
        internalType: "address",
        name: "guardian",
        type: "address",
      },
    ],
    name: "GuardianRemoved",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: false,
        internalType: "uint8",
        name: "version",
        type: "uint8",
      },
    ],
    name: "Initialized",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "address",
        name: "account",
        type: "address",
      },
      {
        indexed: true,
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
      {
        indexed: true,
        internalType: "bytes32",
        name: "hashedCredentialId",
        type: "bytes32",
      },
    ],
    name: "RecoveryDiscarded",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "address",
        name: "account",
        type: "address",
      },
      {
        indexed: true,
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
      {
        indexed: true,
        internalType: "bytes32",
        name: "hashedCredentialId",
        type: "bytes32",
      },
    ],
    name: "RecoveryFinished",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "address",
        name: "account",
        type: "address",
      },
      {
        indexed: true,
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
      {
        indexed: true,
        internalType: "bytes32",
        name: "hashedCredentialId",
        type: "bytes32",
      },
      {
        indexed: false,
        internalType: "address",
        name: "guardian",
        type: "address",
      },
    ],
    name: "RecoveryInitiated",
    type: "event",
  },
  {
    inputs: [],
    name: "REQUEST_DELAY_TIME",
    outputs: [
      {
        internalType: "uint256",
        name: "",
        type: "uint256",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [],
    name: "REQUEST_VALIDITY_TIME",
    outputs: [
      {
        internalType: "uint256",
        name: "",
        type: "uint256",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
      {
        internalType: "address",
        name: "account",
        type: "address",
      },
      {
        internalType: "address",
        name: "guardian",
        type: "address",
      },
    ],
    name: "accountGuardianData",
    outputs: [
      {
        internalType: "address",
        name: "addr",
        type: "address",
      },
      {
        internalType: "bool",
        name: "isReady",
        type: "bool",
      },
      {
        internalType: "uint64",
        name: "addedAt",
        type: "uint64",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
      {
        internalType: "address",
        name: "accountToGuard",
        type: "address",
      },
    ],
    name: "addValidationKey",
    outputs: [
      {
        internalType: "bool",
        name: "",
        type: "bool",
      },
    ],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
    ],
    name: "discardRecovery",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
      {
        internalType: "address",
        name: "account",
        type: "address",
      },
    ],
    name: "getPendingRecoveryData",
    outputs: [
      {
        components: [
          {
            internalType: "bytes32",
            name: "hashedCredentialId",
            type: "bytes32",
          },
          {
            internalType: "bytes32[2]",
            name: "rawPublicKey",
            type: "bytes32[2]",
          },
          {
            internalType: "uint256",
            name: "timestamp",
            type: "uint256",
          },
        ],
        internalType: "struct GuardianRecoveryValidator.RecoveryRequest",
        name: "",
        type: "tuple",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
      {
        internalType: "address",
        name: "guardian",
        type: "address",
      },
    ],
    name: "guardianOf",
    outputs: [
      {
        internalType: "address[]",
        name: "",
        type: "address[]",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
      {
        internalType: "address",
        name: "addr",
        type: "address",
      },
    ],
    name: "guardiansFor",
    outputs: [
      {
        components: [
          {
            internalType: "address",
            name: "addr",
            type: "address",
          },
          {
            internalType: "bool",
            name: "isReady",
            type: "bool",
          },
          {
            internalType: "uint64",
            name: "addedAt",
            type: "uint64",
          },
        ],
        internalType: "struct GuardianRecoveryValidator.Guardian[]",
        name: "",
        type: "tuple[]",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "accountToRecover",
        type: "address",
      },
      {
        internalType: "bytes32",
        name: "hashedCredentialId",
        type: "bytes32",
      },
      {
        internalType: "bytes32[2]",
        name: "rawPublicKey",
        type: "bytes32[2]",
      },
      {
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
    ],
    name: "initRecovery",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "contract WebAuthValidator",
        name: "_webAuthValidator",
        type: "address",
      },
    ],
    name: "initialize",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes",
        name: "",
        type: "bytes",
      },
    ],
    name: "onInstall",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes",
        name: "data",
        type: "bytes",
      },
    ],
    name: "onUninstall",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
      {
        internalType: "address",
        name: "account",
        type: "address",
      },
    ],
    name: "pendingRecoveryData",
    outputs: [
      {
        internalType: "bytes32",
        name: "hashedCredentialId",
        type: "bytes32",
      },
      {
        internalType: "uint256",
        name: "timestamp",
        type: "uint256",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
      {
        internalType: "address",
        name: "newGuardian",
        type: "address",
      },
    ],
    name: "proposeValidationKey",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "hashedOriginDomain",
        type: "bytes32",
      },
      {
        internalType: "address",
        name: "guardianToRemove",
        type: "address",
      },
    ],
    name: "removeValidationKey",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes4",
        name: "interfaceId",
        type: "bytes4",
      },
    ],
    name: "supportsInterface",
    outputs: [
      {
        internalType: "bool",
        name: "",
        type: "bool",
      },
    ],
    stateMutability: "pure",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "",
        type: "bytes32",
      },
      {
        internalType: "bytes",
        name: "",
        type: "bytes",
      },
    ],
    name: "validateSignature",
    outputs: [
      {
        internalType: "bool",
        name: "",
        type: "bool",
      },
    ],
    stateMutability: "pure",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "",
        type: "bytes32",
      },
      {
        components: [
          {
            internalType: "uint256",
            name: "txType",
            type: "uint256",
          },
          {
            internalType: "uint256",
            name: "from",
            type: "uint256",
          },
          {
            internalType: "uint256",
            name: "to",
            type: "uint256",
          },
          {
            internalType: "uint256",
            name: "gasLimit",
            type: "uint256",
          },
          {
            internalType: "uint256",
            name: "gasPerPubdataByteLimit",
            type: "uint256",
          },
          {
            internalType: "uint256",
            name: "maxFeePerGas",
            type: "uint256",
          },
          {
            internalType: "uint256",
            name: "maxPriorityFeePerGas",
            type: "uint256",
          },
          {
            internalType: "uint256",
            name: "paymaster",
            type: "uint256",
          },
          {
            internalType: "uint256",
            name: "nonce",
            type: "uint256",
          },
          {
            internalType: "uint256",
            name: "value",
            type: "uint256",
          },
          {
            internalType: "uint256[4]",
            name: "reserved",
            type: "uint256[4]",
          },
          {
            internalType: "bytes",
            name: "data",
            type: "bytes",
          },
          {
            internalType: "bytes",
            name: "signature",
            type: "bytes",
          },
          {
            internalType: "bytes32[]",
            name: "factoryDeps",
            type: "bytes32[]",
          },
          {
            internalType: "bytes",
            name: "paymasterInput",
            type: "bytes",
          },
          {
            internalType: "bytes",
            name: "reservedDynamic",
            type: "bytes",
          },
        ],
        internalType: "struct Transaction",
        name: "transaction",
        type: "tuple",
      },
    ],
    name: "validateTransaction",
    outputs: [
      {
        internalType: "bool",
        name: "",
        type: "bool",
      },
    ],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [],
    name: "webAuthValidator",
    outputs: [
      {
        internalType: "contract WebAuthValidator",
        name: "",
        type: "address",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
] as const;
