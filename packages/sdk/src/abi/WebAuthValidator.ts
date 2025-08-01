export const WebAuthValidatorAbi = [
  {
    inputs: [],
    name: "WEBAUTHN_ACCOUNT_EXISTS",
    type: "error",
  },
  {
    inputs: [],
    name: "WEBAUTHN_BAD_CREDENTIAL_ID_LENGTH",
    type: "error",
  },
  {
    inputs: [],
    name: "WEBAUTHN_BAD_DOMAIN_LENGTH",
    type: "error",
  },
  {
    inputs: [],
    name: "WEBAUTHN_EMPTY_KEY",
    type: "error",
  },
  {
    inputs: [],
    name: "WEBAUTHN_KEY_EXISTS",
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
    name: "WEBAUTHN_NOT_KEY_OWNER",
    type: "error",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "address",
        name: "keyOwner",
        type: "address",
      },
      {
        indexed: false,
        internalType: "string",
        name: "originDomain",
        type: "string",
      },
      {
        indexed: false,
        internalType: "bytes",
        name: "credentialId",
        type: "bytes",
      },
    ],
    name: "PasskeyCreated",
    type: "event",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "address",
        name: "keyOwner",
        type: "address",
      },
      {
        indexed: false,
        internalType: "string",
        name: "originDomain",
        type: "string",
      },
      {
        indexed: false,
        internalType: "bytes",
        name: "credentialId",
        type: "bytes",
      },
    ],
    name: "PasskeyRemoved",
    type: "event",
  },
  {
    inputs: [
      {
        internalType: "bytes",
        name: "credentialId",
        type: "bytes",
      },
      {
        internalType: "bytes32[2]",
        name: "rawPublicKey",
        type: "bytes32[2]",
      },
      {
        internalType: "string",
        name: "originDomain",
        type: "string",
      },
    ],
    name: "addValidationKey",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "string",
        name: "originDomain",
        type: "string",
      },
      {
        internalType: "bytes",
        name: "credentialId",
        type: "bytes",
      },
      {
        internalType: "address",
        name: "accountAddress",
        type: "address",
      },
    ],
    name: "getAccountKey",
    outputs: [
      {
        internalType: "bytes32[2]",
        name: "",
        type: "bytes32[2]",
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
        internalType: "string",
        name: "originDomain",
        type: "string",
      },
      {
        internalType: "bytes",
        name: "credentialId",
        type: "bytes",
      },
    ],
    name: "registeredAddress",
    outputs: [
      {
        internalType: "address",
        name: "accountAddress",
        type: "address",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes",
        name: "credentialId",
        type: "bytes",
      },
      {
        internalType: "string",
        name: "domain",
        type: "string",
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
    stateMutability: "view",
    type: "function",
  },
] as const;
