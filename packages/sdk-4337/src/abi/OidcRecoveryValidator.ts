export const OidcRecoveryValidatorAbi = [
  {
    inputs: [],
    stateMutability: "nonpayable",
    type: "constructor",
  },
  {
    inputs: [
      {
        internalType: "uint256",
        name: "value",
        type: "uint256",
      },
    ],
    name: "ADDRESS_CAST_OVERFLOW",
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
    inputs: [
      {
        internalType: "bytes32",
        name: "digest",
        type: "bytes32",
      },
    ],
    name: "OIDC_ADDRESS_NOT_FOUND",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "currentAccount",
        type: "address",
      },
    ],
    name: "OIDC_DIGEST_TAKEN",
    type: "error",
  },
  {
    inputs: [],
    name: "OIDC_EMPTY_DIGEST",
    type: "error",
  },
  {
    inputs: [],
    name: "OIDC_EMPTY_ISSUER",
    type: "error",
  },
  {
    inputs: [],
    name: "OIDC_ISSUER_TOO_LONG",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "account",
        type: "address",
      },
    ],
    name: "OIDC_NO_DATA_FOR_ACCOUNT",
    type: "error",
  },
  {
    inputs: [],
    name: "OIDC_NO_RECOVERY_STARTED",
    type: "error",
  },
  {
    inputs: [],
    name: "OIDC_TIME_LIMIT_EXPIRED",
    type: "error",
  },
  {
    inputs: [],
    name: "OIDC_ZERO_KEY_REGISTRY",
    type: "error",
  },
  {
    inputs: [],
    name: "OIDC_ZERO_VERIFIER",
    type: "error",
  },
  {
    inputs: [],
    name: "OIDC_ZERO_WEBAUTH_VALIDATOR",
    type: "error",
  },
  {
    inputs: [],
    name: "OIDC_ZKP_VERIFICATION_FAILED",
    type: "error",
  },
  {
    inputs: [],
    name: "WEBAUTH_VALIDATOR_NOT_INSTALLED",
    type: "error",
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
        indexed: false,
        internalType: "bytes32",
        name: "oidcDigest",
        type: "bytes32",
      },
    ],
    name: "OidcAccountDeleted",
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
        indexed: false,
        internalType: "bytes32",
        name: "oidcDigest",
        type: "bytes32",
      },
      {
        indexed: false,
        internalType: "string",
        name: "iss",
        type: "string",
      },
      {
        indexed: false,
        internalType: "bool",
        name: "isNew",
        type: "bool",
      },
    ],
    name: "OidcAccountUpdated",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "address",
        name: "targetAccount",
        type: "address",
      },
      {
        indexed: false,
        internalType: "bytes32",
        name: "pendingPasskeyHash",
        type: "bytes32",
      },
    ],
    name: "RecoveryCancelled",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "address",
        name: "initiator",
        type: "address",
      },
      {
        indexed: true,
        internalType: "address",
        name: "targetAccount",
        type: "address",
      },
      {
        indexed: false,
        internalType: "bytes32",
        name: "pendingPasskeyHash",
        type: "bytes32",
      },
    ],
    name: "RecoveryStarted",
    type: "event",
  },
  {
    inputs: [],
    name: "MAX_ISS_LENGTH",
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
    name: "PUB_SIGNALS_LENGTH",
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
        name: "oidcDigest",
        type: "bytes32",
      },
      {
        internalType: "string",
        name: "iss",
        type: "string",
      },
    ],
    name: "addOidcAccount",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "digest",
        type: "bytes32",
      },
    ],
    name: "addressForDigest",
    outputs: [
      {
        internalType: "address",
        name: "",
        type: "address",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [],
    name: "cancelRecovery",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [],
    name: "deleteOidcAccount",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "_keyRegistry",
        type: "address",
      },
      {
        internalType: "address",
        name: "_verifier",
        type: "address",
      },
      {
        internalType: "address",
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
    inputs: [],
    name: "keyRegistry",
    outputs: [
      {
        internalType: "contract IOidcKeyRegistry",
        name: "",
        type: "address",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "account",
        type: "address",
      },
    ],
    name: "oidcDataForAddress",
    outputs: [
      {
        components: [
          {
            internalType: "bytes32",
            name: "oidcDigest",
            type: "bytes32",
          },
          {
            internalType: "string",
            name: "iss",
            type: "string",
          },
          {
            internalType: "bool",
            name: "readyToRecover",
            type: "bool",
          },
          {
            internalType: "bytes32",
            name: "pendingPasskeyHash",
            type: "bytes32",
          },
          {
            internalType: "uint256",
            name: "recoveryStartedAt",
            type: "uint256",
          },
          {
            internalType: "uint256",
            name: "recoverNonce",
            type: "uint256",
          },
          {
            internalType: "uint256",
            name: "addedOn",
            type: "uint256",
          },
        ],
        internalType: "struct IOidcRecoveryValidator.OidcData",
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
        internalType: "bytes",
        name: "data",
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
        name: "",
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
        components: [
          {
            components: [
              {
                internalType: "uint256[2]",
                name: "pA",
                type: "uint256[2]",
              },
              {
                internalType: "uint256[2][2]",
                name: "pB",
                type: "uint256[2][2]",
              },
              {
                internalType: "uint256[2]",
                name: "pC",
                type: "uint256[2]",
              },
            ],
            internalType: "struct IOidcRecoveryValidator.ZkProof",
            name: "zkProof",
            type: "tuple",
          },
          {
            internalType: "bytes32",
            name: "kid",
            type: "bytes32",
          },
          {
            internalType: "bytes32",
            name: "pendingPasskeyHash",
            type: "bytes32",
          },
          {
            internalType: "uint256",
            name: "timeLimit",
            type: "uint256",
          },
        ],
        internalType: "struct IOidcRecoveryValidator.StartRecoveryData",
        name: "data",
        type: "tuple",
      },
      {
        internalType: "address",
        name: "targetAccount",
        type: "address",
      },
    ],
    name: "startRecovery",
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
    name: "verifier",
    outputs: [
      {
        internalType: "contract IZkVerifier",
        name: "",
        type: "address",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [],
    name: "webAuthValidator",
    outputs: [
      {
        internalType: "address",
        name: "",
        type: "address",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
] as const;
