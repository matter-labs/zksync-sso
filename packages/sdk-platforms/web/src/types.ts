/**
 * Configuration for zkSync SSO ERC-4337 client
 */
export interface ClientConfig {
  /** RPC URL for the Ethereum provider */
  rpcUrl: string;
  /** Bundler URL for ERC-4337 operations */
  bundlerUrl: string;
  /** Smart contract addresses */
  contracts: {
    /** EntryPoint contract address */
    entryPoint: string;
    /** Account factory contract address */
    accountFactory: string;
  };
}

/**
 * Transaction call data
 */
export interface CallData {
  /** Target contract address */
  to: string;
  /** Transaction data (hex string) */
  data: string;
  /** ETH value to send (in wei as string) */
  value: string;
}

/**
 * User operation request
 */
export interface UserOperationRequest {
  /** Smart contract account address */
  account: string;
  /** Array of calls to execute */
  calls: CallData[];
}

/**
 * Error information from the WASM module
 */
export interface ErrorInfo {
  /** Error message */
  message: string;
  /** Error type/kind */
  kind: string;
}

/**
 * Result type for async operations
 */
export type Result<T> = Promise<T>;

/**
 * WASM module initialization options
 */
export interface InitOptions {
  /** Custom WASM module URL (optional) */
  wasmUrl?: string;
  /** Enable debug logging (optional) */
  debug?: boolean;
}

/**
 * Paymaster parameters for sponsoring user operations
 */
export interface PaymasterConfig {
  /** Paymaster contract address (0x-prefixed hex string) */
  address: string;
  /** Additional paymaster data (0x-prefixed hex string, optional) */
  data?: string;
  /** Verification gas limit for paymaster (decimal string, optional) */
  verificationGasLimit?: string;
  /** Post-operation gas limit for paymaster (decimal string, optional) */
  postOpGasLimit?: string;
}

/**
 * Configuration for sending user operations
 */
export interface SendUserOperationParams {
  /** RPC URL for provider */
  rpcUrl: string;
  /** Bundler URL */
  bundlerUrl: string;
  /** EntryPoint contract address */
  entryPointAddress: string;
  /** Smart account address */
  accountAddress: string;
  /** Encoded call data (0x-prefixed hex string) */
  callData: string;
  /** Private key for signing (0x-prefixed hex string) */
  privateKeyHex: string;
  /** Validator module address */
  validatorAddress: string;
  /** Optional paymaster configuration */
  paymaster?: PaymasterConfig;
}
