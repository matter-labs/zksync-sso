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

// ===== Session Types (Phase 2) =====

/** Types of usage limits for fees and transfers */
export type LimitType = "unlimited" | "lifetime" | "allowance";

/** Usage limit definition */
export interface UsageLimit {
  /** Limit type */
  limitType: LimitType;
  /** Total limit amount (wei) */
  limit: bigint;
  /** Period in seconds for allowance type (optional) */
  period?: number;
}

/** Transfer permission within a session */
export interface TransferSpec {
  /** Target address */
  to: `0x${string}`;
  /** Max ETH per transfer (wei) */
  valueLimit: bigint;
  /** Optional override of limit type for this transfer */
  limitType?: LimitType;
  /** Optional period for allowance type */
  period?: number;
}

/** Session configuration passed from web app */
export interface SessionConfig {
  /** Session key address */
  signer: `0x${string}`;
  /** Expiration timestamp (unix seconds) */
  expiresAt: number | bigint;
  /** Fee usage limit */
  feeLimit: UsageLimit;
  /** Allowed transfers */
  transfers: TransferSpec[];
}
