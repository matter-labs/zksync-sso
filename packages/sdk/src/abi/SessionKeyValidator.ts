export const SessionKeyValidatorAbi = [{
  type: "function",
  name: "createSession",
  inputs: [{
    name: "sessionSpec",
    type: "tuple",
    internalType: "struct SessionLib.SessionSpec",
    components: [{
      name: "signer",
      type: "address",
      internalType: "address",
    }, {
      name: "expiresAt",
      type: "uint48",
      internalType: "uint48",
    }, {
      name: "feeLimit",
      type: "tuple",
      internalType: "struct SessionLib.UsageLimit",
      components: [{
        name: "limitType",
        type: "uint8",
        internalType: "enum SessionLib.LimitType",
      }, {
        name: "limit",
        type: "uint256",
        internalType: "uint256",
      }, {
        name: "period",
        type: "uint48",
        internalType: "uint48",
      }],
    }, {
      name: "callPolicies",
      type: "tuple[]",
      internalType: "struct SessionLib.CallSpec[]",
      components: [{
        name: "target",
        type: "address",
        internalType: "address",
      }, {
        name: "selector",
        type: "bytes4",
        internalType: "bytes4",
      }, {
        name: "maxValuePerUse",
        type: "uint256",
        internalType: "uint256",
      }, {
        name: "valueLimit",
        type: "tuple",
        internalType: "struct SessionLib.UsageLimit",
        components: [{
          name: "limitType",
          type: "uint8",
          internalType: "enum SessionLib.LimitType",
        }, {
          name: "limit",
          type: "uint256",
          internalType: "uint256",
        }, {
          name: "period",
          type: "uint48",
          internalType: "uint48",
        }],
      }, {
        name: "constraints",
        type: "tuple[]",
        internalType: "struct SessionLib.Constraint[]",
        components: [{
          name: "condition",
          type: "uint8",
          internalType: "enum SessionLib.Condition",
        }, {
          name: "index",
          type: "uint64",
          internalType: "uint64",
        }, {
          name: "refValue",
          type: "bytes32",
          internalType: "bytes32",
        }, {
          name: "limit",
          type: "tuple",
          internalType: "struct SessionLib.UsageLimit",
          components: [{
            name: "limitType",
            type: "uint8",
            internalType: "enum SessionLib.LimitType",
          }, {
            name: "limit",
            type: "uint256",
            internalType: "uint256",
          }, {
            name: "period",
            type: "uint48",
            internalType: "uint48",
          }],
        }],
      }],
    }, {
      name: "transferPolicies",
      type: "tuple[]",
      internalType: "struct SessionLib.TransferSpec[]",
      components: [{
        name: "target",
        type: "address",
        internalType: "address",
      }, {
        name: "maxValuePerUse",
        type: "uint256",
        internalType: "uint256",
      }, {
        name: "valueLimit",
        type: "tuple",
        internalType: "struct SessionLib.UsageLimit",
        components: [{
          name: "limitType",
          type: "uint8",
          internalType: "enum SessionLib.LimitType",
        }, {
          name: "limit",
          type: "uint256",
          internalType: "uint256",
        }, {
          name: "period",
          type: "uint48",
          internalType: "uint48",
        }],
      }],
    }],
  }, {
    name: "proof",
    type: "bytes",
    internalType: "bytes",
  }],
  outputs: [],
  stateMutability: "nonpayable",
}, {
  type: "function",
  name: "isInitialized",
  inputs: [{
    name: "smartAccount",
    type: "address",
    internalType: "address",
  }],
  outputs: [{
    name: "",
    type: "bool",
    internalType: "bool",
  }],
  stateMutability: "view",
}, {
  type: "function",
  name: "isModuleType",
  inputs: [{
    name: "moduleTypeId",
    type: "uint256",
    internalType: "uint256",
  }],
  outputs: [{
    name: "",
    type: "bool",
    internalType: "bool",
  }],
  stateMutability: "pure",
}, {
  type: "function",
  name: "isValidSignatureWithSender",
  inputs: [{
    name: "",
    type: "address",
    internalType: "address",
  }, {
    name: "",
    type: "bytes32",
    internalType: "bytes32",
  }, {
    name: "",
    type: "bytes",
    internalType: "bytes",
  }],
  outputs: [{
    name: "",
    type: "bytes4",
    internalType: "bytes4",
  }],
  stateMutability: "pure",
}, {
  type: "function",
  name: "onInstall",
  inputs: [{
    name: "data",
    type: "bytes",
    internalType: "bytes",
  }],
  outputs: [],
  stateMutability: "nonpayable",
}, {
  type: "function",
  name: "onUninstall",
  inputs: [{
    name: "data",
    type: "bytes",
    internalType: "bytes",
  }],
  outputs: [],
  stateMutability: "nonpayable",
}, {
  type: "function",
  name: "revokeKey",
  inputs: [{
    name: "sessionHash",
    type: "bytes32",
    internalType: "bytes32",
  }],
  outputs: [],
  stateMutability: "nonpayable",
}, {
  type: "function",
  name: "revokeKeys",
  inputs: [{
    name: "sessionHashes",
    type: "bytes32[]",
    internalType: "bytes32[]",
  }],
  outputs: [],
  stateMutability: "nonpayable",
}, {
  type: "function",
  name: "sessionSigner",
  inputs: [{
    name: "signer",
    type: "address",
    internalType: "address",
  }],
  outputs: [{
    name: "sessionHash",
    type: "bytes32",
    internalType: "bytes32",
  }],
  stateMutability: "view",
}, {
  type: "function",
  name: "sessionState",
  inputs: [{
    name: "account",
    type: "address",
    internalType: "address",
  }, {
    name: "spec",
    type: "tuple",
    internalType: "struct SessionLib.SessionSpec",
    components: [{
      name: "signer",
      type: "address",
      internalType: "address",
    }, {
      name: "expiresAt",
      type: "uint48",
      internalType: "uint48",
    }, {
      name: "feeLimit",
      type: "tuple",
      internalType: "struct SessionLib.UsageLimit",
      components: [{
        name: "limitType",
        type: "uint8",
        internalType: "enum SessionLib.LimitType",
      }, {
        name: "limit",
        type: "uint256",
        internalType: "uint256",
      }, {
        name: "period",
        type: "uint48",
        internalType: "uint48",
      }],
    }, {
      name: "callPolicies",
      type: "tuple[]",
      internalType: "struct SessionLib.CallSpec[]",
      components: [{
        name: "target",
        type: "address",
        internalType: "address",
      }, {
        name: "selector",
        type: "bytes4",
        internalType: "bytes4",
      }, {
        name: "maxValuePerUse",
        type: "uint256",
        internalType: "uint256",
      }, {
        name: "valueLimit",
        type: "tuple",
        internalType: "struct SessionLib.UsageLimit",
        components: [{
          name: "limitType",
          type: "uint8",
          internalType: "enum SessionLib.LimitType",
        }, {
          name: "limit",
          type: "uint256",
          internalType: "uint256",
        }, {
          name: "period",
          type: "uint48",
          internalType: "uint48",
        }],
      }, {
        name: "constraints",
        type: "tuple[]",
        internalType: "struct SessionLib.Constraint[]",
        components: [{
          name: "condition",
          type: "uint8",
          internalType: "enum SessionLib.Condition",
        }, {
          name: "index",
          type: "uint64",
          internalType: "uint64",
        }, {
          name: "refValue",
          type: "bytes32",
          internalType: "bytes32",
        }, {
          name: "limit",
          type: "tuple",
          internalType: "struct SessionLib.UsageLimit",
          components: [{
            name: "limitType",
            type: "uint8",
            internalType: "enum SessionLib.LimitType",
          }, {
            name: "limit",
            type: "uint256",
            internalType: "uint256",
          }, {
            name: "period",
            type: "uint48",
            internalType: "uint48",
          }],
        }],
      }],
    }, {
      name: "transferPolicies",
      type: "tuple[]",
      internalType: "struct SessionLib.TransferSpec[]",
      components: [{
        name: "target",
        type: "address",
        internalType: "address",
      }, {
        name: "maxValuePerUse",
        type: "uint256",
        internalType: "uint256",
      }, {
        name: "valueLimit",
        type: "tuple",
        internalType: "struct SessionLib.UsageLimit",
        components: [{
          name: "limitType",
          type: "uint8",
          internalType: "enum SessionLib.LimitType",
        }, {
          name: "limit",
          type: "uint256",
          internalType: "uint256",
        }, {
          name: "period",
          type: "uint48",
          internalType: "uint48",
        }],
      }],
    }],
  }],
  outputs: [{
    name: "",
    type: "tuple",
    internalType: "struct SessionLib.SessionState",
    components: [{
      name: "status",
      type: "uint8",
      internalType: "enum SessionLib.Status",
    }, {
      name: "feesRemaining",
      type: "uint256",
      internalType: "uint256",
    }, {
      name: "transferValue",
      type: "tuple[]",
      internalType: "struct SessionLib.LimitState[]",
      components: [{
        name: "remaining",
        type: "uint256",
        internalType: "uint256",
      }, {
        name: "target",
        type: "address",
        internalType: "address",
      }, {
        name: "selector",
        type: "bytes4",
        internalType: "bytes4",
      }, {
        name: "index",
        type: "uint256",
        internalType: "uint256",
      }],
    }, {
      name: "callValue",
      type: "tuple[]",
      internalType: "struct SessionLib.LimitState[]",
      components: [{
        name: "remaining",
        type: "uint256",
        internalType: "uint256",
      }, {
        name: "target",
        type: "address",
        internalType: "address",
      }, {
        name: "selector",
        type: "bytes4",
        internalType: "bytes4",
      }, {
        name: "index",
        type: "uint256",
        internalType: "uint256",
      }],
    }, {
      name: "callParams",
      type: "tuple[]",
      internalType: "struct SessionLib.LimitState[]",
      components: [{
        name: "remaining",
        type: "uint256",
        internalType: "uint256",
      }, {
        name: "target",
        type: "address",
        internalType: "address",
      }, {
        name: "selector",
        type: "bytes4",
        internalType: "bytes4",
      }, {
        name: "index",
        type: "uint256",
        internalType: "uint256",
      }],
    }],
  }],
  stateMutability: "view",
}, {
  type: "function",
  name: "sessionStatus",
  inputs: [{
    name: "account",
    type: "address",
    internalType: "address",
  }, {
    name: "sessionHash",
    type: "bytes32",
    internalType: "bytes32",
  }],
  outputs: [{
    name: "",
    type: "uint8",
    internalType: "enum SessionLib.Status",
  }],
  stateMutability: "view",
}, {
  type: "function",
  name: "supportsInterface",
  inputs: [{
    name: "interfaceId",
    type: "bytes4",
    internalType: "bytes4",
  }],
  outputs: [{
    name: "",
    type: "bool",
    internalType: "bool",
  }],
  stateMutability: "pure",
}, {
  type: "function",
  name: "validateUserOp",
  inputs: [{
    name: "userOp",
    type: "tuple",
    internalType: "struct PackedUserOperation",
    components: [{
      name: "sender",
      type: "address",
      internalType: "address",
    }, {
      name: "nonce",
      type: "uint256",
      internalType: "uint256",
    }, {
      name: "initCode",
      type: "bytes",
      internalType: "bytes",
    }, {
      name: "callData",
      type: "bytes",
      internalType: "bytes",
    }, {
      name: "accountGasLimits",
      type: "bytes32",
      internalType: "bytes32",
    }, {
      name: "preVerificationGas",
      type: "uint256",
      internalType: "uint256",
    }, {
      name: "gasFees",
      type: "bytes32",
      internalType: "bytes32",
    }, {
      name: "paymasterAndData",
      type: "bytes",
      internalType: "bytes",
    }, {
      name: "signature",
      type: "bytes",
      internalType: "bytes",
    }],
  }, {
    name: "userOpHash",
    type: "bytes32",
    internalType: "bytes32",
  }],
  outputs: [{
    name: "",
    type: "uint256",
    internalType: "uint256",
  }],
  stateMutability: "nonpayable",
}, {
  type: "event",
  name: "SessionCreated",
  inputs: [{
    name: "account",
    type: "address",
    indexed: true,
    internalType: "address",
  }, {
    name: "sessionHash",
    type: "bytes32",
    indexed: true,
    internalType: "bytes32",
  }, {
    name: "sessionSpec",
    type: "tuple",
    indexed: false,
    internalType: "struct SessionLib.SessionSpec",
    components: [{
      name: "signer",
      type: "address",
      internalType: "address",
    }, {
      name: "expiresAt",
      type: "uint48",
      internalType: "uint48",
    }, {
      name: "feeLimit",
      type: "tuple",
      internalType: "struct SessionLib.UsageLimit",
      components: [{
        name: "limitType",
        type: "uint8",
        internalType: "enum SessionLib.LimitType",
      }, {
        name: "limit",
        type: "uint256",
        internalType: "uint256",
      }, {
        name: "period",
        type: "uint48",
        internalType: "uint48",
      }],
    }, {
      name: "callPolicies",
      type: "tuple[]",
      internalType: "struct SessionLib.CallSpec[]",
      components: [{
        name: "target",
        type: "address",
        internalType: "address",
      }, {
        name: "selector",
        type: "bytes4",
        internalType: "bytes4",
      }, {
        name: "maxValuePerUse",
        type: "uint256",
        internalType: "uint256",
      }, {
        name: "valueLimit",
        type: "tuple",
        internalType: "struct SessionLib.UsageLimit",
        components: [{
          name: "limitType",
          type: "uint8",
          internalType: "enum SessionLib.LimitType",
        }, {
          name: "limit",
          type: "uint256",
          internalType: "uint256",
        }, {
          name: "period",
          type: "uint48",
          internalType: "uint48",
        }],
      }, {
        name: "constraints",
        type: "tuple[]",
        internalType: "struct SessionLib.Constraint[]",
        components: [{
          name: "condition",
          type: "uint8",
          internalType: "enum SessionLib.Condition",
        }, {
          name: "index",
          type: "uint64",
          internalType: "uint64",
        }, {
          name: "refValue",
          type: "bytes32",
          internalType: "bytes32",
        }, {
          name: "limit",
          type: "tuple",
          internalType: "struct SessionLib.UsageLimit",
          components: [{
            name: "limitType",
            type: "uint8",
            internalType: "enum SessionLib.LimitType",
          }, {
            name: "limit",
            type: "uint256",
            internalType: "uint256",
          }, {
            name: "period",
            type: "uint48",
            internalType: "uint48",
          }],
        }],
      }],
    }, {
      name: "transferPolicies",
      type: "tuple[]",
      internalType: "struct SessionLib.TransferSpec[]",
      components: [{
        name: "target",
        type: "address",
        internalType: "address",
      }, {
        name: "maxValuePerUse",
        type: "uint256",
        internalType: "uint256",
      }, {
        name: "valueLimit",
        type: "tuple",
        internalType: "struct SessionLib.UsageLimit",
        components: [{
          name: "limitType",
          type: "uint8",
          internalType: "enum SessionLib.LimitType",
        }, {
          name: "limit",
          type: "uint256",
          internalType: "uint256",
        }, {
          name: "period",
          type: "uint48",
          internalType: "uint48",
        }],
      }],
    }],
  }],
  anonymous: false,
}, {
  type: "event",
  name: "SessionRevoked",
  inputs: [{
    name: "account",
    type: "address",
    indexed: true,
    internalType: "address",
  }, {
    name: "sessionHash",
    type: "bytes32",
    indexed: true,
    internalType: "bytes32",
  }],
  anonymous: false,
}, {
  type: "error",
  name: "AllowanceExceeded",
  inputs: [{
    name: "allowance",
    type: "uint256",
    internalType: "uint256",
  }, {
    name: "maxAllowance",
    type: "uint256",
    internalType: "uint256",
  }, {
    name: "period",
    type: "uint64",
    internalType: "uint64",
  }],
}, {
  type: "error",
  name: "AlreadyInitialized",
  inputs: [{
    name: "smartAccount",
    type: "address",
    internalType: "address",
  }],
}, {
  type: "error",
  name: "CallPolicyBanned",
  inputs: [{
    name: "target",
    type: "address",
    internalType: "address",
  }, {
    name: "selector",
    type: "bytes4",
    internalType: "bytes4",
  }],
}, {
  type: "error",
  name: "CallPolicyViolated",
  inputs: [{
    name: "target",
    type: "address",
    internalType: "address",
  }, {
    name: "selector",
    type: "bytes4",
    internalType: "bytes4",
  }],
}, {
  type: "error",
  name: "ConditionViolated",
  inputs: [{
    name: "param",
    type: "bytes32",
    internalType: "bytes32",
  }, {
    name: "refValue",
    type: "bytes32",
    internalType: "bytes32",
  }, {
    name: "condition",
    type: "uint8",
    internalType: "uint8",
  }],
}, {
  type: "error",
  name: "InvalidCallType",
  inputs: [{
    name: "callType",
    type: "bytes1",
    internalType: "bytes1",
  }, {
    name: "expected",
    type: "bytes1",
    internalType: "bytes1",
  }],
}, {
  type: "error",
  name: "InvalidDataLength",
  inputs: [{
    name: "actualLength",
    type: "uint256",
    internalType: "uint256",
  }, {
    name: "expectedMinimumLength",
    type: "uint256",
    internalType: "uint256",
  }],
}, {
  type: "error",
  name: "InvalidNonceKey",
  inputs: [{
    name: "nonceKey",
    type: "uint192",
    internalType: "uint192",
  }, {
    name: "expectedNonceKey",
    type: "uint192",
    internalType: "uint192",
  }],
}, {
  type: "error",
  name: "InvalidTargetAddress",
  inputs: [{
    name: "target",
    type: "address",
    internalType: "address",
  }],
}, {
  type: "error",
  name: "InvalidTopLevelSelector",
  inputs: [{
    name: "selector",
    type: "bytes4",
    internalType: "bytes4",
  }, {
    name: "expected",
    type: "bytes4",
    internalType: "bytes4",
  }],
}, {
  type: "error",
  name: "LifetimeUsageExceeded",
  inputs: [{
    name: "lifetimeUsage",
    type: "uint256",
    internalType: "uint256",
  }, {
    name: "maxUsage",
    type: "uint256",
    internalType: "uint256",
  }],
}, {
  type: "error",
  name: "MaxValueExceeded",
  inputs: [{
    name: "usedValue",
    type: "uint256",
    internalType: "uint256",
  }, {
    name: "maxValuePerUse",
    type: "uint256",
    internalType: "uint256",
  }],
}, {
  type: "error",
  name: "NotInitialized",
  inputs: [{
    name: "smartAccount",
    type: "address",
    internalType: "address",
  }],
}, {
  type: "error",
  name: "SessionAlreadyExists",
  inputs: [{
    name: "sessionHash",
    type: "bytes32",
    internalType: "bytes32",
  }],
}, {
  type: "error",
  name: "SessionExpiresTooSoon",
  inputs: [{
    name: "expiresAt",
    type: "uint256",
    internalType: "uint256",
  }],
}, {
  type: "error",
  name: "SessionNotActive",
  inputs: [],
}, {
  type: "error",
  name: "SignerAlreadyUsed",
  inputs: [{
    name: "signer",
    type: "address",
    internalType: "address",
  }],
}, {
  type: "error",
  name: "TransferPolicyViolated",
  inputs: [{
    name: "target",
    type: "address",
    internalType: "address",
  }],
}, {
  type: "error",
  name: "UnlimitedFees",
  inputs: [],
}, {
  type: "error",
  name: "ZeroSigner",
  inputs: [],
}] as const;
