import type { Request, Response } from "express";
import type { PrividiumSiweChain } from "prividium/siwe";
import { type Address, type Chain, createPublicClient, createWalletClient, type Hex, http, parseEther, type Transport } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { waitForTransactionReceipt } from "viem/actions";
import { getAccountAddressFromLogs, prepareDeploySmartAccount } from "zksync-sso-4337/client";

import { env, EOA_VALIDATOR_ADDRESS, FACTORY_ADDRESS, getChain, GUARDIAN_EXECUTOR_ADDRESS, prividiumConfig, SESSION_VALIDATOR_ADDRESS, WEBAUTHN_VALIDATOR_ADDRESS } from "../config.js";
import { deployAccountSchema } from "../schemas.js";
import { addAddressToUser, getAdminAuthService, whitelistContract } from "../services/prividium/index.js";

type DeployAccountRequest = {
  chainId: number;
  credentialId: Hex;
  credentialPublicKey: { x: Hex; y: Hex };
  originDomain: string;
  userId?: string;
  eoaSigners?: Address[];
};

/**
 * Checks if an error is a Prividium session expiry error.
 * PrividiumSessionError is not re-exported from the SDK, so we check the message
 * in the error's cause chain (viem wraps it in HttpRequestError).
 */
function isPrividiumSessionError(error: unknown): boolean {
  if (!(error instanceof Error)) return false;
  const message = error.cause instanceof Error ? error.cause.message : error.message;
  return message.includes("No session started or previous session expired");
}

type DeploymentResult = {
  deployedAddress: Address;
};

/**
 * Core deployment logic: creates clients, checks balance, deploys account.
 * Separated from the handler to allow retry on auth errors.
 */
async function executeDeployment(body: DeployAccountRequest, adminSdk: PrividiumSiweChain | undefined, chain: Chain): Promise<DeploymentResult> {
  // Create transport - use SDK transport if enabled, otherwise direct RPC
  const transport: Transport = adminSdk
    ? adminSdk.transport // SDK provides authenticated transport
    : http(env.RPC_URL);

  // Create clients
  const publicClient = createPublicClient({
    chain,
    transport,
  });

  const deployerAccount = privateKeyToAccount(env.DEPLOYER_PRIVATE_KEY as Hex);
  const walletClient = createWalletClient({
    account: deployerAccount,
    chain,
    transport,
  });

  // Check deployer balance
  const balance = await publicClient.getBalance({ address: deployerAccount.address });
  const minBalance = parseEther("0.01"); // Minimum balance required for deployment
  if (balance < minBalance) {
    console.error(`Deployer balance too low: ${balance.toString()} < ${minBalance.toString()}`);
    throw new DeploymentError("Deployer doesn't have enough balance to cover deployment");
  }

  // Prepare deployment transaction
  const executorModulesToInstall = GUARDIAN_EXECUTOR_ADDRESS ? [GUARDIAN_EXECUTOR_ADDRESS as Address] : [];

  const { transaction, accountId } = prepareDeploySmartAccount({
    contracts: {
      factory: FACTORY_ADDRESS as Address,
      eoaValidator: EOA_VALIDATOR_ADDRESS as Address,
      webauthnValidator: WEBAUTHN_VALIDATOR_ADDRESS as Address,
      sessionValidator: SESSION_VALIDATOR_ADDRESS as Address,
    },
    passkeySigners: [
      {
        credentialId: body.credentialId,
        publicKey: body.credentialPublicKey,
        originDomain: body.originDomain,
      },
    ],
    eoaSigners: body.eoaSigners,
    userId: body.userId,
    installSessionValidator: true,
    executorModules: executorModulesToInstall,
  });

  console.log("Deploying account with ID:", accountId);

  // Send transaction
  let txHash: Hex;
  try {
    txHash = await walletClient.sendTransaction({
      to: transaction.to,
      data: transaction.data,
    });
  } catch (error) {
    console.error("Transaction send failed:", error);
    const errorMessage = error instanceof Error ? error.message : "Unknown error";
    if (errorMessage.includes("insufficient funds")) {
      throw new DeploymentError("Insufficient balance to cover deployment");
    }
    throw error;
  }

  console.log("Transaction sent:", txHash);

  // Wait for transaction receipt
  const receipt = await waitForTransactionReceipt(publicClient, {
    hash: txHash,
  });

  // Check if transaction was successful
  if (receipt.status === "reverted") {
    throw new DeploymentError("Deployment reverted");
  }

  // Extract deployed address from logs
  const deployedAddress = getAccountAddressFromLogs(receipt.logs);

  console.log("Account deployed at:", deployedAddress);

  return { deployedAddress };
}

class DeploymentError extends Error {
  constructor(message: string) {
    super(message);
  }
}

// Deploy account endpoint
export const deployAccountHandler = async (req: Request, res: Response): Promise<void> => {
  try {
    // Validate request body
    const validationResult = deployAccountSchema.safeParse(req.body);
    if (!validationResult.success) {
      res.status(400).json({
        error: `Invalid parameters: ${validationResult.error.errors.map((e) => e.message).join(", ")}`,
      });
      return;
    }

    const body = req.body as DeployAccountRequest;

    // Get chain from request
    const chain = getChain(body.chainId);

    // Get SDK instance if in Prividium mode (provides authenticated transport and headers)
    let adminSdk: PrividiumSiweChain | undefined;
    if (prividiumConfig.enabled && req.prividiumUser) {
      try {
        adminSdk = getAdminAuthService().getSdkInstance();
      } catch (error) {
        console.error("Admin authentication failed:", error);
        res.status(500).json({
          error: "Admin authentication failed",
        });
        return;
      }
    }

    // Execute deployment with retry on Prividium session expiry
    let result: DeploymentResult;
    try {
      result = await executeDeployment(body, adminSdk, chain);
    } catch (error) {
      if (isPrividiumSessionError(error) && adminSdk) {
        // Reauthenticate and retry once
        console.log("Prividium session expired during deployment, retrying after reauthentication...");
        await getAdminAuthService().reauthenticate();
        result = await executeDeployment(body, adminSdk, chain);
      } else if (error instanceof DeploymentError) {
        res.status(500).json({ error: error.message });
        return;
      } else {
        throw error;
      }
    }

    // Prividium post-deployment steps (all blocking)
    if (prividiumConfig.enabled && req.prividiumUser && adminSdk) {
      // Get auth headers from SDK
      const authHeaders = adminSdk.getAuthHeaders();
      if (!authHeaders) {
        console.error("Failed to get auth headers");
        res.status(500).json({
          error: "Authentication error",
        });
        return;
      }

      // Step 1: Whitelist the contract with template (blocking)
      try {
        await whitelistContract(
          result.deployedAddress,
          prividiumConfig.templateKey,
          authHeaders,
          prividiumConfig.apiUrl,
        );
      } catch (error) {
        console.error("Failed to whitelist contract:", error);
        res.status(500).json({
          error: "Failed to whitelist contract",
        });
        return;
      }

      // Step 2: Associate address with user (blocking)
      try {
        await addAddressToUser(
          req.prividiumUser.userId,
          [result.deployedAddress],
          authHeaders,
          prividiumConfig.apiUrl,
        );
      } catch (error) {
        console.error("Failed to associate address with user:", error);
        res.status(500).json({
          error: "Failed to associate address with user",
        });
        return;
      }
    }

    // Return success response
    res.json({ address: result.deployedAddress });
  } catch (error) {
    console.error("Deployment error:", error);
    res.status(500).json({
      error: "Internal server error",
    });
  }
};
