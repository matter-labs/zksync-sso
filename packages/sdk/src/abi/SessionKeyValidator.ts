export const SessionKeyValidatorAbi = [
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
        internalType: "bytes",
        name: "input",
        type: "bytes",
      },
    ],
    name: "INVALID_PAYMASTER_INPUT",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "notInitialized",
        type: "address",
      },
    ],
    name: "NOT_FROM_INITIALIZED_ACCOUNT",
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
        internalType: "uint256",
        name: "allowance",
        type: "uint256",
      },
      {
        internalType: "uint256",
        name: "maxAllowance",
        type: "uint256",
      },
      {
        internalType: "uint64",
        name: "period",
        type: "uint64",
      },
    ],
    name: "SESSION_ALLOWANCE_EXCEEDED",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "sessionHash",
        type: "bytes32",
      },
    ],
    name: "SESSION_ALREADY_EXISTS",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "target",
        type: "address",
      },
      {
        internalType: "bytes4",
        name: "selector",
        type: "bytes4",
      },
    ],
    name: "SESSION_CALL_POLICY_VIOLATED",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "bytes32",
        name: "param",
        type: "bytes32",
      },
      {
        internalType: "bytes32",
        name: "refValue",
        type: "bytes32",
      },
      {
        internalType: "uint8",
        name: "condition",
        type: "uint8",
      },
    ],
    name: "SESSION_CONDITION_FAILED",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "uint256",
        name: "expiresAt",
        type: "uint256",
      },
    ],
    name: "SESSION_EXPIRES_TOO_SOON",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "uint256",
        name: "actualLength",
        type: "uint256",
      },
      {
        internalType: "uint256",
        name: "expectedMinimumLength",
        type: "uint256",
      },
    ],
    name: "SESSION_INVALID_DATA_LENGTH",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "recovered",
        type: "address",
      },
      {
        internalType: "address",
        name: "expected",
        type: "address",
      },
    ],
    name: "SESSION_INVALID_SIGNER",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "uint256",
        name: "lifetimeUsage",
        type: "uint256",
      },
      {
        internalType: "uint256",
        name: "maxUsage",
        type: "uint256",
      },
    ],
    name: "SESSION_LIFETIME_USAGE_EXCEEDED",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "uint256",
        name: "usedValue",
        type: "uint256",
      },
      {
        internalType: "uint256",
        name: "maxValuePerUse",
        type: "uint256",
      },
    ],
    name: "SESSION_MAX_VALUE_EXCEEDED",
    type: "error",
  },
  {
    inputs: [],
    name: "SESSION_NOT_ACTIVE",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "target",
        type: "address",
      },
    ],
    name: "SESSION_TRANSFER_POLICY_VIOLATED",
    type: "error",
  },
  {
    inputs: [],
    name: "SESSION_UNLIMITED_FEES",
    type: "error",
  },
  {
    inputs: [],
    name: "SESSION_ZERO_SIGNER",
    type: "error",
  },
  {
    inputs: [
      {
        internalType: "uint256",
        name: "openSessions",
        type: "uint256",
      },
    ],
    name: "UNINSTALL_WITH_OPEN_SESSIONS",
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
        name: "sessionHash",
        type: "bytes32",
      },
      {
        components: [
          {
            internalType: "address",
            name: "signer",
            type: "address",
          },
          {
            internalType: "uint256",
            name: "expiresAt",
            type: "uint256",
          },
          {
            components: [
              {
                internalType: "enum SessionLib.LimitType",
                name: "limitType",
                type: "uint8",
              },
              {
                internalType: "uint256",
                name: "limit",
                type: "uint256",
              },
              {
                internalType: "uint256",
                name: "period",
                type: "uint256",
              },
            ],
            internalType: "struct SessionLib.UsageLimit",
            name: "feeLimit",
            type: "tuple",
          },
          {
            components: [
              {
                internalType: "address",
                name: "target",
                type: "address",
              },
              {
                internalType: "bytes4",
                name: "selector",
                type: "bytes4",
              },
              {
                internalType: "uint256",
                name: "maxValuePerUse",
                type: "uint256",
              },
              {
                components: [
                  {
                    internalType: "enum SessionLib.LimitType",
                    name: "limitType",
                    type: "uint8",
                  },
                  {
                    internalType: "uint256",
                    name: "limit",
                    type: "uint256",
                  },
                  {
                    internalType: "uint256",
                    name: "period",
                    type: "uint256",
                  },
                ],
                internalType: "struct SessionLib.UsageLimit",
                name: "valueLimit",
                type: "tuple",
              },
              {
                components: [
                  {
                    internalType: "enum SessionLib.Condition",
                    name: "condition",
                    type: "uint8",
                  },
                  {
                    internalType: "uint64",
                    name: "index",
                    type: "uint64",
                  },
                  {
                    internalType: "bytes32",
                    name: "refValue",
                    type: "bytes32",
                  },
                  {
                    components: [
                      {
                        internalType: "enum SessionLib.LimitType",
                        name: "limitType",
                        type: "uint8",
                      },
                      {
                        internalType: "uint256",
                        name: "limit",
                        type: "uint256",
                      },
                      {
                        internalType: "uint256",
                        name: "period",
                        type: "uint256",
                      },
                    ],
                    internalType: "struct SessionLib.UsageLimit",
                    name: "limit",
                    type: "tuple",
                  },
                ],
                internalType: "struct SessionLib.Constraint[]",
                name: "constraints",
                type: "tuple[]",
              },
            ],
            internalType: "struct SessionLib.CallSpec[]",
            name: "callPolicies",
            type: "tuple[]",
          },
          {
            components: [
              {
                internalType: "address",
                name: "target",
                type: "address",
              },
              {
                internalType: "uint256",
                name: "maxValuePerUse",
                type: "uint256",
              },
              {
                components: [
                  {
                    internalType: "enum SessionLib.LimitType",
                    name: "limitType",
                    type: "uint8",
                  },
                  {
                    internalType: "uint256",
                    name: "limit",
                    type: "uint256",
                  },
                  {
                    internalType: "uint256",
                    name: "period",
                    type: "uint256",
                  },
                ],
                internalType: "struct SessionLib.UsageLimit",
                name: "valueLimit",
                type: "tuple",
              },
            ],
            internalType: "struct SessionLib.TransferSpec[]",
            name: "transferPolicies",
            type: "tuple[]",
          },
        ],
        indexed: false,
        internalType: "struct SessionLib.SessionSpec",
        name: "sessionSpec",
        type: "tuple",
      },
    ],
    name: "SessionCreated",
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
        name: "sessionHash",
        type: "bytes32",
      },
    ],
    name: "SessionRevoked",
    type: "event",
  },
  {
    inputs: [
      {
        components: [
          {
            internalType: "address",
            name: "signer",
            type: "address",
          },
          {
            internalType: "uint256",
            name: "expiresAt",
            type: "uint256",
          },
          {
            components: [
              {
                internalType: "enum SessionLib.LimitType",
                name: "limitType",
                type: "uint8",
              },
              {
                internalType: "uint256",
                name: "limit",
                type: "uint256",
              },
              {
                internalType: "uint256",
                name: "period",
                type: "uint256",
              },
            ],
            internalType: "struct SessionLib.UsageLimit",
            name: "feeLimit",
            type: "tuple",
          },
          {
            components: [
              {
                internalType: "address",
                name: "target",
                type: "address",
              },
              {
                internalType: "bytes4",
                name: "selector",
                type: "bytes4",
              },
              {
                internalType: "uint256",
                name: "maxValuePerUse",
                type: "uint256",
              },
              {
                components: [
                  {
                    internalType: "enum SessionLib.LimitType",
                    name: "limitType",
                    type: "uint8",
                  },
                  {
                    internalType: "uint256",
                    name: "limit",
                    type: "uint256",
                  },
                  {
                    internalType: "uint256",
                    name: "period",
                    type: "uint256",
                  },
                ],
                internalType: "struct SessionLib.UsageLimit",
                name: "valueLimit",
                type: "tuple",
              },
              {
                components: [
                  {
                    internalType: "enum SessionLib.Condition",
                    name: "condition",
                    type: "uint8",
                  },
                  {
                    internalType: "uint64",
                    name: "index",
                    type: "uint64",
                  },
                  {
                    internalType: "bytes32",
                    name: "refValue",
                    type: "bytes32",
                  },
                  {
                    components: [
                      {
                        internalType: "enum SessionLib.LimitType",
                        name: "limitType",
                        type: "uint8",
                      },
                      {
                        internalType: "uint256",
                        name: "limit",
                        type: "uint256",
                      },
                      {
                        internalType: "uint256",
                        name: "period",
                        type: "uint256",
                      },
                    ],
                    internalType: "struct SessionLib.UsageLimit",
                    name: "limit",
                    type: "tuple",
                  },
                ],
                internalType: "struct SessionLib.Constraint[]",
                name: "constraints",
                type: "tuple[]",
              },
            ],
            internalType: "struct SessionLib.CallSpec[]",
            name: "callPolicies",
            type: "tuple[]",
          },
          {
            components: [
              {
                internalType: "address",
                name: "target",
                type: "address",
              },
              {
                internalType: "uint256",
                name: "maxValuePerUse",
                type: "uint256",
              },
              {
                components: [
                  {
                    internalType: "enum SessionLib.LimitType",
                    name: "limitType",
                    type: "uint8",
                  },
                  {
                    internalType: "uint256",
                    name: "limit",
                    type: "uint256",
                  },
                  {
                    internalType: "uint256",
                    name: "period",
                    type: "uint256",
                  },
                ],
                internalType: "struct SessionLib.UsageLimit",
                name: "valueLimit",
                type: "tuple",
              },
            ],
            internalType: "struct SessionLib.TransferSpec[]",
            name: "transferPolicies",
            type: "tuple[]",
          },
        ],
        internalType: "struct SessionLib.SessionSpec",
        name: "sessionSpec",
        type: "tuple",
      },
    ],
    name: "createSession",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "smartAccount",
        type: "address",
      },
    ],
    name: "isInitialized",
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
        internalType: "bytes32",
        name: "sessionHash",
        type: "bytes32",
      },
    ],
    name: "revokeKey",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "bytes32[]",
        name: "sessionHashes",
        type: "bytes32[]",
      },
    ],
    name: "revokeKeys",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      {
        internalType: "address",
        name: "account",
        type: "address",
      },
      {
        components: [
          {
            internalType: "address",
            name: "signer",
            type: "address",
          },
          {
            internalType: "uint256",
            name: "expiresAt",
            type: "uint256",
          },
          {
            components: [
              {
                internalType: "enum SessionLib.LimitType",
                name: "limitType",
                type: "uint8",
              },
              {
                internalType: "uint256",
                name: "limit",
                type: "uint256",
              },
              {
                internalType: "uint256",
                name: "period",
                type: "uint256",
              },
            ],
            internalType: "struct SessionLib.UsageLimit",
            name: "feeLimit",
            type: "tuple",
          },
          {
            components: [
              {
                internalType: "address",
                name: "target",
                type: "address",
              },
              {
                internalType: "bytes4",
                name: "selector",
                type: "bytes4",
              },
              {
                internalType: "uint256",
                name: "maxValuePerUse",
                type: "uint256",
              },
              {
                components: [
                  {
                    internalType: "enum SessionLib.LimitType",
                    name: "limitType",
                    type: "uint8",
                  },
                  {
                    internalType: "uint256",
                    name: "limit",
                    type: "uint256",
                  },
                  {
                    internalType: "uint256",
                    name: "period",
                    type: "uint256",
                  },
                ],
                internalType: "struct SessionLib.UsageLimit",
                name: "valueLimit",
                type: "tuple",
              },
              {
                components: [
                  {
                    internalType: "enum SessionLib.Condition",
                    name: "condition",
                    type: "uint8",
                  },
                  {
                    internalType: "uint64",
                    name: "index",
                    type: "uint64",
                  },
                  {
                    internalType: "bytes32",
                    name: "refValue",
                    type: "bytes32",
                  },
                  {
                    components: [
                      {
                        internalType: "enum SessionLib.LimitType",
                        name: "limitType",
                        type: "uint8",
                      },
                      {
                        internalType: "uint256",
                        name: "limit",
                        type: "uint256",
                      },
                      {
                        internalType: "uint256",
                        name: "period",
                        type: "uint256",
                      },
                    ],
                    internalType: "struct SessionLib.UsageLimit",
                    name: "limit",
                    type: "tuple",
                  },
                ],
                internalType: "struct SessionLib.Constraint[]",
                name: "constraints",
                type: "tuple[]",
              },
            ],
            internalType: "struct SessionLib.CallSpec[]",
            name: "callPolicies",
            type: "tuple[]",
          },
          {
            components: [
              {
                internalType: "address",
                name: "target",
                type: "address",
              },
              {
                internalType: "uint256",
                name: "maxValuePerUse",
                type: "uint256",
              },
              {
                components: [
                  {
                    internalType: "enum SessionLib.LimitType",
                    name: "limitType",
                    type: "uint8",
                  },
                  {
                    internalType: "uint256",
                    name: "limit",
                    type: "uint256",
                  },
                  {
                    internalType: "uint256",
                    name: "period",
                    type: "uint256",
                  },
                ],
                internalType: "struct SessionLib.UsageLimit",
                name: "valueLimit",
                type: "tuple",
              },
            ],
            internalType: "struct SessionLib.TransferSpec[]",
            name: "transferPolicies",
            type: "tuple[]",
          },
        ],
        internalType: "struct SessionLib.SessionSpec",
        name: "spec",
        type: "tuple",
      },
    ],
    name: "sessionState",
    outputs: [
      {
        components: [
          {
            internalType: "enum SessionLib.Status",
            name: "status",
            type: "uint8",
          },
          {
            internalType: "uint256",
            name: "feesRemaining",
            type: "uint256",
          },
          {
            components: [
              {
                internalType: "uint256",
                name: "remaining",
                type: "uint256",
              },
              {
                internalType: "address",
                name: "target",
                type: "address",
              },
              {
                internalType: "bytes4",
                name: "selector",
                type: "bytes4",
              },
              {
                internalType: "uint256",
                name: "index",
                type: "uint256",
              },
            ],
            internalType: "struct SessionLib.LimitState[]",
            name: "transferValue",
            type: "tuple[]",
          },
          {
            components: [
              {
                internalType: "uint256",
                name: "remaining",
                type: "uint256",
              },
              {
                internalType: "address",
                name: "target",
                type: "address",
              },
              {
                internalType: "bytes4",
                name: "selector",
                type: "bytes4",
              },
              {
                internalType: "uint256",
                name: "index",
                type: "uint256",
              },
            ],
            internalType: "struct SessionLib.LimitState[]",
            name: "callValue",
            type: "tuple[]",
          },
          {
            components: [
              {
                internalType: "uint256",
                name: "remaining",
                type: "uint256",
              },
              {
                internalType: "address",
                name: "target",
                type: "address",
              },
              {
                internalType: "bytes4",
                name: "selector",
                type: "bytes4",
              },
              {
                internalType: "uint256",
                name: "index",
                type: "uint256",
              },
            ],
            internalType: "struct SessionLib.LimitState[]",
            name: "callParams",
            type: "tuple[]",
          },
        ],
        internalType: "struct SessionLib.SessionState",
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
        internalType: "address",
        name: "account",
        type: "address",
      },
      {
        internalType: "bytes32",
        name: "sessionHash",
        type: "bytes32",
      },
    ],
    name: "sessionStatus",
    outputs: [
      {
        internalType: "enum SessionLib.Status",
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
] as const;
