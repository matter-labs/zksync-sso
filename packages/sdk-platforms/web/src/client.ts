import type { CallData, ClientConfig, UserOperationRequest } from "./types";

// WASM-generated constructor types
export interface WasmClientConstructor {
  new(config: WasmConfigInstance, privateKey: string): WasmClientInstance;
}

export interface WasmConfigConstructor {
  new(rpcUrl: string, bundlerUrl: string, contracts: WasmContractsInstance): WasmConfigInstance;
}

export interface WasmContractsConstructor {
  new(entryPoint: string, accountFactory: string): WasmContractsInstance;
}

export interface WasmCallConstructor {
  new(to: string, data: string, value: string): WasmCallInstance;
}

export interface WasmSendCallsRequestConstructor {
  new(account: string, calls: WasmCallInstance[]): WasmSendCallsRequestInstance;
}

// WASM instance types
export interface WasmClientInstance {
  send_user_operation(request: WasmSendCallsRequestInstance): Promise<string>;
  config: WasmConfigInstance;
}

export interface WasmConfigInstance {
  // WASM config instance methods/properties
  readonly __wasmConfigBrand: unique symbol;
}

export interface WasmContractsInstance {
  // WASM contracts instance methods/properties
  readonly __wasmContractsBrand: unique symbol;
}

export interface WasmCallInstance {
  // WASM call instance methods/properties
  readonly __wasmCallBrand: unique symbol;
}

export interface WasmSendCallsRequestInstance {
  // WASM send calls request instance methods/properties
  readonly __wasmSendCallsRequestBrand: unique symbol;
}

// These will be set by the bundler/node entry points
let WasmClient: WasmClientConstructor;
let WasmConfig: WasmConfigConstructor;
let WasmContracts: WasmContractsConstructor;
let WasmCall: WasmCallConstructor;
let WasmSendCallsRequest: WasmSendCallsRequestConstructor;

// Function to initialize WASM bindings - called by bundler/node modules
export function setWasmBindings(bindings: {
  Client: WasmClientConstructor;
  Config: WasmConfigConstructor;
  Contracts: WasmContractsConstructor;
  Call: WasmCallConstructor;
  SendCallsRequest: WasmSendCallsRequestConstructor;
}) {
  WasmClient = bindings.Client;
  WasmConfig = bindings.Config;
  WasmContracts = bindings.Contracts;
  WasmCall = bindings.Call;
  WasmSendCallsRequest = bindings.SendCallsRequest;
}

/**
 * High-level TypeScript wrapper for the zkSync SSO ERC-4337 client
 */
export class ZkSyncSsoClient {
  private client: WasmClientInstance;

  /**
   * Create a new zkSync SSO client
   * @param config - Client configuration
   * @param privateKey - Private key for signing (hex string with or without 0x prefix)
   */
  constructor(config: ClientConfig, privateKey: string) {
    // Ensure private key has 0x prefix
    const formattedKey = privateKey.startsWith("0x") ? privateKey : `0x${privateKey}`;

    // Create WASM contract configuration
    const contracts = new WasmContracts(
      config.contracts.entryPoint,
      config.contracts.accountFactory,
    );

    // Create WASM client configuration
    const wasmConfig = new WasmConfig(
      config.rpcUrl,
      config.bundlerUrl,
      contracts,
    );

    // Initialize the WASM client
    this.client = new WasmClient(wasmConfig, formattedKey);
  }

  /**
   * Send a user operation to the bundler
   * @param request - User operation request containing account and calls
   * @returns Promise resolving to the user operation hash
   */
  async sendUserOperation(request: UserOperationRequest): Promise<string> {
    // Convert TypeScript call data to WASM format
    const calls = request.calls.map((call: CallData) =>
      new WasmCall(call.to, call.data, call.value),
    );

    // Create WASM request object
    const wasmRequest = new WasmSendCallsRequest(request.account, calls);

    // Execute the user operation
    return await this.client.send_user_operation(wasmRequest);
  }

  /**
   * Get the underlying WASM client instance (for advanced use cases)
   * @returns The raw WASM client object
   */
  getWasmClient(): WasmClientInstance {
    return this.client;
  }
}

/**
 * Utility functions for common operations
 */
export class ZkSyncSsoUtils {
  /**
   * Convert bytes to hex string
   * @param bytes - Byte array
   * @returns Hex string with 0x prefix
   */
  static bytesToHex(bytes: Uint8Array): string {
    // Will use WASM function when available
    return `0x${Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join("")}`;
  }

  /**
   * Convert hex string to bytes
   * @param hex - Hex string (with or without 0x prefix)
   * @returns Byte array
   */
  static hexToBytes(hex: string): Uint8Array {
    const cleanHex = hex.startsWith("0x") ? hex.slice(2) : hex;
    const bytes = new Uint8Array(cleanHex.length / 2);
    for (let i = 0; i < cleanHex.length; i += 2) {
      bytes[i / 2] = parseInt(cleanHex.substr(i, 2), 16);
    }
    return bytes;
  }

  /**
   * Validate an Ethereum address
   * @param address - Address to validate
   * @returns True if address is valid
   */
  static isValidAddress(address: string): boolean {
    return /^0x[a-fA-F0-9]{40}$/.test(address);
  }

  /**
   * Validate a private key
   * @param privateKey - Private key to validate (hex string)
   * @returns True if private key is valid
   */
  static isValidPrivateKey(privateKey: string): boolean {
    const cleanKey = privateKey.startsWith("0x") ? privateKey.slice(2) : privateKey;
    return /^[a-fA-F0-9]{64}$/.test(cleanKey);
  }
}

export default ZkSyncSsoClient;
