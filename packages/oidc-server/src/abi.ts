export const abi = [
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
    name: "MAX_KEYS",
    outputs: [
      {
        internalType: "uint8",
        name: "",
        type: "uint8",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "uint256",
        name: "",
        type: "uint256",
      },
    ],
    name: "OIDCKeys",
    outputs: [
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
        internalType: "bytes",
        name: "n",
        type: "bytes",
      },
      {
        internalType: "bytes",
        name: "e",
        type: "bytes",
      },
    ],
    stateMutability: "view",
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
            internalType: "bytes",
            name: "n",
            type: "bytes",
          },
          {
            internalType: "bytes",
            name: "e",
            type: "bytes",
          },
        ],
        internalType: "struct OidcKeyRegistry.Key",
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
            internalType: "bytes",
            name: "n",
            type: "bytes",
          },
          {
            internalType: "bytes",
            name: "e",
            type: "bytes",
          },
        ],
        internalType: "struct OidcKeyRegistry.Key[]",
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
            internalType: "bytes",
            name: "n",
            type: "bytes",
          },
          {
            internalType: "bytes",
            name: "e",
            type: "bytes",
          },
        ],
        internalType: "struct OidcKeyRegistry.Key",
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
    name: "keyIndex",
    outputs: [
      {
        internalType: "uint8",
        name: "",
        type: "uint8",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [],
    name: "merkleRoot",
    outputs: [
      {
        internalType: "bytes32",
        name: "",
        type: "bytes32",
      },
    ],
    stateMutability: "view",
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
            internalType: "bytes",
            name: "n",
            type: "bytes",
          },
          {
            internalType: "bytes",
            name: "e",
            type: "bytes",
          },
        ],
        internalType: "struct OidcKeyRegistry.Key",
        name: "key",
        type: "tuple",
      },
      {
        internalType: "bytes32[]",
        name: "proof",
        type: "bytes32[]",
      },
    ],
    name: "verifyKey",
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
];
