export const abi = [
  {
    "type": "constructor",
    "inputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "MAX_KEYS",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "uint8",
        "internalType": "uint8"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "OIDCKeys",
    "inputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "",
        "type": "uint256",
        "internalType": "uint256"
      }
    ],
    "outputs": [
      {
        "name": "kid",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "n",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "e",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "getKey",
    "inputs": [
      {
        "name": "issHash",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "kid",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "tuple",
        "internalType": "struct OidcKeyRegistry.Key",
        "components": [
          {
            "name": "kid",
            "type": "bytes32",
            "internalType": "bytes32"
          },
          {
            "name": "n",
            "type": "bytes",
            "internalType": "bytes"
          },
          {
            "name": "e",
            "type": "bytes",
            "internalType": "bytes"
          }
        ]
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "hashIssuer",
    "inputs": [
      {
        "name": "iss",
        "type": "string",
        "internalType": "string"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "stateMutability": "pure"
  },
  {
    "type": "function",
    "name": "initialize",
    "inputs": [],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "keyIndexes",
    "inputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "uint8",
        "internalType": "uint8"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "owner",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "renounceOwnership",
    "inputs": [],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "setKey",
    "inputs": [
      {
        "name": "issHash",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "key",
        "type": "tuple",
        "internalType": "struct OidcKeyRegistry.Key",
        "components": [
          {
            "name": "kid",
            "type": "bytes32",
            "internalType": "bytes32"
          },
          {
            "name": "n",
            "type": "bytes",
            "internalType": "bytes"
          },
          {
            "name": "e",
            "type": "bytes",
            "internalType": "bytes"
          }
        ]
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "setKeys",
    "inputs": [
      {
        "name": "issHash",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "keys",
        "type": "tuple[]",
        "internalType": "struct OidcKeyRegistry.Key[]",
        "components": [
          {
            "name": "kid",
            "type": "bytes32",
            "internalType": "bytes32"
          },
          {
            "name": "n",
            "type": "bytes",
            "internalType": "bytes"
          },
          {
            "name": "e",
            "type": "bytes",
            "internalType": "bytes"
          }
        ]
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "transferOwnership",
    "inputs": [
      {
        "name": "newOwner",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "event",
    "name": "Initialized",
    "inputs": [
      {
        "name": "version",
        "type": "uint8",
        "indexed": false,
        "internalType": "uint8"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "OwnershipTransferred",
    "inputs": [
      {
        "name": "previousOwner",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      },
      {
        "name": "newOwner",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  }
]
