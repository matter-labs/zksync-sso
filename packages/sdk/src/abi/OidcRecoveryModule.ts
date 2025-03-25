export const OidcRecoveryModuleAbi = [
  {
    inputs: [],
    stateMutability: "nonpayable",
    type: "constructor",
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
    name: "OidcKeyUpdated",
    type: "event",
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
        internalType: "address",
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
            name: "recoverNonce",
            type: "uint256",
          },
        ],
        internalType: "struct OidcRecoveryValidator.OidcData",
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
            internalType: "struct OidcRecoveryValidator.ZkProof",
            name: "zkProof",
            type: "tuple",
          },
          {
            internalType: "bytes32",
            name: "issHash",
            type: "bytes32",
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
        ],
        internalType: "struct OidcRecoveryValidator.StartRecoveryData",
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
        name: "signedHash",
        type: "bytes32",
      },
      {
        internalType: "bytes",
        name: "signature",
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
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "signedHash",
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
