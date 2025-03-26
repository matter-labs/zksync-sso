export const AAFactoryAbi = [
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
        internalType: "address",
        name: "account",
        type: "address",
      },
    ],
    name: "ACCOUNT_ALREADY_EXISTS",
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
        internalType: "bytes32",
        name: "uniqueAccountId",
        type: "bytes32",
      },
    ],
    name: "AccountCreated",
    type: "event",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "accountId",
        type: "bytes32",
      },
    ],
    name: "accountMappings",
    outputs: [
      {
        internalType: "address",
        name: "deployedAccount",
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
        name: "uniqueId",
        type: "bytes32",
      },
      {
        internalType: "bytes[]",
        name: "initialValidators",
        type: "bytes[]",
      },
      {
        internalType: "address[]",
        name: "initialK1Owners",
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
] as const;
