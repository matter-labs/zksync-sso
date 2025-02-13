export const FactoryAbi = [
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "_beaconProxyBytecodeHash",
        type: "bytes32",
      },
      {
        internalType: "address",
        name: "_beacon",
        type: "address",
      },
    ],
    stateMutability: "nonpayable",
    type: "constructor",
  },
  {
    inputs: [
      {
        internalType: "string",
        name: "uniqueAccountId",
        type: "string",
      },
      {
        internalType: "address",
        name: "accountAddress",
        type: "address",
      },
    ],
    name: "AccountAlreadyRegistered",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "string",
        name: "uniqueAccountId",
        type: "string",
      },
      {
        internalType: "address",
        name: "accountAddress",
        type: "address",
      },
    ],
    name: "AccountNotRegistered",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "string",
        name: "uniqueAccountId",
        type: "string",
      },
      {
        internalType: "address",
        name: "accountAddress",
        type: "address",
      },
    ],
    name: "AccountUsedForRecovery",
    type: "error",
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: "address",
        name: "accountAddress",
        type: "address",
      },
      {
        indexed: false,
        internalType: "string",
        name: "uniqueAccountId",
        type: "string",
      },
    ],
    name: "AccountCreated",
    type: "event",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "",
        type: "address",
      },
    ],
    name: "accountIds",
    outputs: [
      {
        internalType: "string",
        name: "",
        type: "string",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "string",
        name: "",
        type: "string",
      },
    ],
    name: "accountMappings",
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
    name: "beacon",
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
    name: "beaconProxyBytecodeHash",
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
    inputs: [
      {
        internalType: "bytes32",
        name: "_salt",
        type: "bytes32",
      },
      {
        internalType: "string",
        name: "_uniqueAccountId",
        type: "string",
      },
      {
        internalType: "bytes[]",
        name: "_initialValidators",
        type: "bytes[]",
      },
      {
        internalType: "address[]",
        name: "_initialK1Owners",
        type: "address[]",
      },
    ],
    name: "deployProxySsoAccount",
    outputs: [
      {
        internalType: "address",
        name: "accountAddress",
        type: "address",
      },
    ],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [],
    name: "getEncodedBeacon",
    outputs: [
      {
        internalType: "bytes",
        name: "",
        type: "bytes",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "string",
        name: "",
        type: "string",
      },
    ],
    name: "recoveryAccountIds",
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
        internalType: "string",
        name: "_uniqueAccountId",
        type: "string",
      },
      {
        internalType: "address",
        name: "_accountAddress",
        type: "address",
      },
    ],
    name: "registerAccount",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "string",
        name: "_uniqueAccountId",
        type: "string",
      },
      {
        internalType: "address",
        name: "_accountAddress",
        type: "address",
      },
    ],
    name: "registerRecoveryBlockedAccount",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "string",
        name: "_uniqueAccountId",
        type: "string",
      },
      {
        internalType: "address",
        name: "_accountAddress",
        type: "address",
      },
    ],
    name: "unregisterAccount",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "string",
        name: "_uniqueAccountId",
        type: "string",
      },
      {
        internalType: "address",
        name: "_accountAddress",
        type: "address",
      },
    ],
    name: "unregisterRecoveryBlockedAccount",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
] as const;
