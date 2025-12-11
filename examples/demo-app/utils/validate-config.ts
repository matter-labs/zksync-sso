/**
 * Validates that the auth-server-API is using the correct contract addresses
 * by comparing them with the local contracts configuration
 */

export interface ContractConfig {
  factory: string;
  webauthnValidator: string;
  eoaValidator: string;
  sessionValidator: string;
  entryPoint: string;
  chainId: number;
}

export interface ValidationResult {
  valid: boolean;
  errors: string[];
  apiConfig?: ContractConfig;
  localConfig?: ContractConfig;
}

/**
 * Fetches the auth-server-API configuration
 */
export async function fetchAuthServerConfig(apiUrl: string): Promise<ContractConfig> {
  const response = await fetch(`${apiUrl}/api/config`);
  if (!response.ok) {
    throw new Error(`Failed to fetch auth-server config: ${response.statusText}`);
  }
  const data = await response.json();

  console.log("📡 Raw API response:", JSON.stringify(data, null, 2));

  // Extract contracts with fallback for undefined values
  const config = {
    factory: data.contracts?.factory || data.factory,
    webauthnValidator: data.contracts?.webauthnValidator || data.webauthnValidator,
    eoaValidator: data.contracts?.eoaValidator || data.eoaValidator,
    sessionValidator: data.contracts?.sessionValidator || data.sessionValidator,
    entryPoint: data.contracts?.entryPoint || data.entryPoint,
    chainId: parseInt(data.chainId || data.contracts?.chainId || "1337"),
  };

  console.log("🔍 Parsed config:", JSON.stringify(config, null, 2));

  return config;
}

/**
 * Validates that the auth-server-API and local configs match
 */
export async function validateContractConfig(
  apiUrl: string,
  localConfig: ContractConfig,
): Promise<ValidationResult> {
  const errors: string[] = [];

  try {
    const apiConfig = await fetchAuthServerConfig(apiUrl);

    // Compare addresses (case-insensitive)
    const normalize = (addr: string) => addr.toLowerCase();

    if (normalize(apiConfig.factory) !== normalize(localConfig.factory)) {
      errors.push(
        `Factory address mismatch:\n  API: ${apiConfig.factory}\n  Local: ${localConfig.factory}`,
      );
    }

    if (normalize(apiConfig.webauthnValidator) !== normalize(localConfig.webauthnValidator)) {
      errors.push(
        `WebAuthn Validator mismatch:\n  API: ${apiConfig.webauthnValidator}\n  Local: ${localConfig.webauthnValidator}`,
      );
    }

    if (normalize(apiConfig.eoaValidator) !== normalize(localConfig.eoaValidator)) {
      errors.push(
        `EOA Validator mismatch:\n  API: ${apiConfig.eoaValidator}\n  Local: ${localConfig.eoaValidator}`,
      );
    }

    if (normalize(apiConfig.sessionValidator) !== normalize(localConfig.sessionValidator)) {
      errors.push(
        `Session Validator mismatch:\n  API: ${apiConfig.sessionValidator}\n  Local: ${localConfig.sessionValidator}`,
      );
    }

    if (normalize(apiConfig.entryPoint) !== normalize(localConfig.entryPoint)) {
      errors.push(
        `EntryPoint address mismatch:\n  API: ${apiConfig.entryPoint}\n  Local: ${localConfig.entryPoint}`,
      );
    }

    if (apiConfig.chainId !== localConfig.chainId) {
      errors.push(`Chain ID mismatch:\n  API: ${apiConfig.chainId}\n  Local: ${localConfig.chainId}`);
    }

    return {
      valid: errors.length === 0,
      errors,
      apiConfig,
      localConfig,
    };
  } catch (error) {
    errors.push(`Failed to validate config: ${error instanceof Error ? error.message : String(error)}`);
    return {
      valid: false,
      errors,
      localConfig,
    };
  }
}

/**
 * Logs validation results to console with colors
 */
export function logValidationResult(result: ValidationResult): void {
  if (result.valid) {
    console.log("✅ Contract configuration validated successfully");
    console.log("API and local configs match:");
    if (result.apiConfig) {
      console.log("  Factory:", result.apiConfig.factory);
      console.log("  WebAuthn Validator:", result.apiConfig.webauthnValidator);
      console.log("  Chain ID:", result.apiConfig.chainId);
    }
  } else {
    console.error("❌ Contract configuration mismatch detected!");
    console.error("This may cause AA24 signature errors and deployment failures.");
    console.error("\nErrors:");
    result.errors.forEach((error) => console.error("  -", error));

    if (result.apiConfig && result.localConfig) {
      console.error("\n💡 Solution: Restart the dev server to sync contract addresses");
      console.error("   Run: pnpm nx dev:erc4337 demo-app");
    }
  }
}
