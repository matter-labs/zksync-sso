export const OidcKeyRegistryAbi = [
  {
    inputs: [],
    stateMutability: "nonpayable",
    type: "constructor",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "kid",
        type: "bytes32",
      },
    ],
    name: "OIDC_EVEN_RSA_MODULUS",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "expectedIssuerHash",
        type: "bytes32",
      },
      {
        internalType: "bytes32",
        name: "actualIssuerHash",
        type: "bytes32",
      },
    ],
    name: "OIDC_ISSUER_HASH_MISMATCH",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "uint256",
        name: "count",
        type: "uint256",
      },
    ],
    name: "OIDC_KEY_COUNT_LIMIT_EXCEEDED",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "kid",
        type: "bytes32",
      },
      {
        internalType: "bytes32",
        name: "issuerHash",
        type: "bytes32",
      },
    ],
    name: "OIDC_KEY_ID_ALREADY_EXISTS",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "issuerHash",
        type: "bytes32",
      },
      {
        internalType: "bytes32",
        name: "kid",
        type: "bytes32",
      },
    ],
    name: "OIDC_KEY_NOT_FOUND",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "kid",
        type: "bytes32",
      },
      {
        internalType: "uint256",
        name: "chunkIndex",
        type: "uint256",
      },
      {
        internalType: "uint256",
        name: "chunkValue",
        type: "uint256",
      },
    ],
    name: "OIDC_MODULUS_CHUNK_TOO_LARGE",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "uint256",
        name: "index",
        type: "uint256",
      },
    ],
    name: "OIDC_ZERO_KEY_ID",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "kid",
        type: "bytes32",
      },
    ],
    name: "OIDC_ZERO_MODULUS",
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
        internalType: "bytes32",
        name: "issHash",
        type: "bytes32",
      },
      {
        indexed: true,
        internalType: "bytes32",
        name: "kid",
        type: "bytes32",
      },
      {
        indexed: false,
        internalType: "uint256[17]",
        name: "n",
        type: "uint256[17]",
      },
    ],
    name: "KeyAdded",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "bytes32",
        name: "issHash",
        type: "bytes32",
      },
      {
        indexed: true,
        internalType: "bytes32",
        name: "kid",
        type: "bytes32",
      },
    ],
    name: "KeyDeleted",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "address",
        name: "previousOwner",
        type: "address",
      },
      {
        indexed: true,
        internalType: "address",
        name: "newOwner",
        type: "address",
      },
    ],
    name: "OwnershipTransferStarted",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "address",
        name: "previousOwner",
        type: "address",
      },
      {
        indexed: true,
        internalType: "address",
        name: "newOwner",
        type: "address",
      },
    ],
    name: "OwnershipTransferred",
    type: "event",
  },
  {
    inputs: [],
    name: "CIRCOM_BIGINT_CHUNKS",
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
    name: "CIRCOM_BIGINT_CHUNK_SIZE",
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
    name: "MAX_KEYS",
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
    name: "acceptOwnership",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        components: [
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
            internalType: "uint256[17]",
            name: "rsaModulus",
            type: "uint256[17]",
          },
        ],
        internalType: "struct IOidcKeyRegistry.Key",
        name: "newKey",
        type: "tuple",
      },
    ],
    name: "addKey",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        components: [
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
            internalType: "uint256[17]",
            name: "rsaModulus",
            type: "uint256[17]",
          },
        ],
        internalType: "struct IOidcKeyRegistry.Key[]",
        name: "newKeys",
        type: "tuple[]",
      },
    ],
    name: "addKeys",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
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
    ],
    name: "deleteKey",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
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
    ],
    name: "getKey",
    outputs: [
      {
        components: [
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
            internalType: "uint256[17]",
            name: "rsaModulus",
            type: "uint256[17]",
          },
        ],
        internalType: "struct IOidcKeyRegistry.Key",
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
        name: "issHash",
        type: "bytes32",
      },
    ],
    name: "getKeys",
    outputs: [
      {
        components: [
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
            internalType: "uint256[17]",
            name: "rsaModulus",
            type: "uint256[17]",
          },
        ],
        internalType: "struct IOidcKeyRegistry.Key[8]",
        name: "",
        type: "tuple[8]",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "string",
        name: "iss",
        type: "string",
      },
    ],
    name: "hashIssuer",
    outputs: [
      {
        internalType: "bytes32",
        name: "",
        type: "bytes32",
      },
    ],
    stateMutability: "pure",
    type: "function",
  },
  {
    inputs: [],
    name: "initialize",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [],
    name: "owner",
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
    name: "pendingOwner",
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
    name: "renounceOwnership",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "newOwner",
        type: "address",
      },
    ],
    name: "transferOwnership",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
] as const;
