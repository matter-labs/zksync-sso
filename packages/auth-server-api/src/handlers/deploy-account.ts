import type { Request, Response } from "express";
import { type Address, createPublicClient, createWalletClient, type Hex, http } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { getTransactionCount, waitForTransactionReceipt } from "viem/actions";
import { getAccountAddressFromLogs, prepareDeploySmartAccount } from "zksync-sso-4337/client";

import { env, EOA_VALIDATOR_ADDRESS, FACTORY_ADDRESS, getChain, SESSION_VALIDATOR_ADDRESS, WEBAUTHN_VALIDATOR_ADDRESS } from "../config.js";
import { deployAccountSchema } from "../schemas.js";

type DeployAccountRequest = {
  chainId: number;
  credentialId: Hex;
  credentialPublicKey: { x: Hex; y: Hex };
  originDomain: string;
  userId?: string;
  eoaSigners?: Address[];
};

// Simple in-memory nonce manager for concurrent request handling
let noncePromise: Promise<number> | null = null;
let cachedNonce: number | null = null;

async function getNextNonce(client: ReturnType<typeof createPublicClient>, address: Address): Promise<number> {
  // If there's an ongoing nonce fetch, wait for it
  if (noncePromise) {
    await noncePromise;
  }

  // Create a new promise for this nonce operation
  noncePromise = (async () => {
    if (cachedNonce === null) {
      // First time or after reset - fetch from chain
      cachedNonce = await getTransactionCount(client, { address });
    } else {
      // Increment the cached nonce
      cachedNonce++;
    }
    return cachedNonce;
  })();

  try {
    const nonce = await noncePromise;
    return nonce;
  } finally {
    noncePromise = null;
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

    // Create clients
    const publicClient = createPublicClient({
      chain,
      transport: http(env.RPC_URL),
    });

    const deployerAccount = privateKeyToAccount(env.DEPLOYER_PRIVATE_KEY as Hex);
    const walletClient = createWalletClient({
      account: deployerAccount,
      chain,
      transport: http(env.RPC_URL),
    });

    // Check deployer balance
    const balance = await publicClient.getBalance({ address: deployerAccount.address });
    const minBalance = BigInt("100000000000000000"); // 0.1 ETH minimum
    if (balance < minBalance) {
      res.status(500).json({
        error: `Insufficient balance to cover deployment. Current: ${balance.toString()}, Required: ${minBalance.toString()}`,
      });
      return;
    }

    // Prepare deployment transaction
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
    });

    console.log("🚀 Deployment configuration:");
    console.log("  - Account ID:", accountId);
    console.log("  - Factory:", FACTORY_ADDRESS);
    console.log("  - WebAuthn Validator:", WEBAUTHN_VALIDATOR_ADDRESS);
    console.log("  - Origin Domain:", body.originDomain);
    console.log("  - Credential ID:", body.credentialId.slice(0, 20) + "...");
    console.log("  - Public Key X:", body.credentialPublicKey.x.slice(0, 20) + "...");
    console.log("  - Public Key Y:", body.credentialPublicKey.y.slice(0, 20) + "...");

    // Get next nonce for this deployer
    const nonce = await getNextNonce(publicClient, deployerAccount.address);
    console.log("  - Nonce:", nonce);

    // Send transaction
    let txHash: Hex;
    try {
      txHash = await walletClient.sendTransaction({
        to: transaction.to,
        data: transaction.data,
        nonce,
      });
      console.log("✅ Transaction sent:", txHash);
    } catch (error) {
      console.error("❌ Transaction send failed:", error);
      const errorMessage = error instanceof Error ? error.message : "Unknown error";
      if (errorMessage.includes("insufficient funds")) {
        res.status(500).json({
          error: "Insufficient balance to cover deployment",
        });
        return;
      }
      res.status(500).json({
        error: `Transaction failed: ${errorMessage}`,
      });
      return;
    }

    // Wait for transaction receipt
    let receipt;
    try {
      receipt = await waitForTransactionReceipt(publicClient, {
        hash: txHash,
      });
      console.log("📋 Receipt received, status:", receipt.status);
    } catch (error) {
      console.error("❌ Transaction receipt failed:", error);
      const errorMessage = error instanceof Error ? error.message : "Unknown error";
      res.status(500).json({
        error: `Transaction failed to be mined: ${errorMessage}`,
      });
      return;
    }

    // Check if transaction was successful
    if (receipt.status === "reverted") {
      console.error("❌ Deployment reverted");
      res.status(500).json({
        error: "Deployment reverted: Transaction execution failed",
      });
      return;
    }

    // Extract deployed address from logs
    let deployedAddress: Address;
    try {
      deployedAddress = getAccountAddressFromLogs(receipt.logs);
      console.log("✅ Account deployed at:", deployedAddress);
    } catch (error) {
      console.error("Failed to extract address from logs:", error);
      const errorMessage = error instanceof Error ? error.message : "";
      res.status(500).json({
        error: `Deployment failed: Could not extract account address from logs. ${errorMessage}`,
      });
      return;
    }

    console.log("Account deployed at:", deployedAddress);

    // Return success response
    res.json({ address: deployedAddress });
  } catch (error) {
    console.error("Deployment error:", error);
    const errorMessage = error instanceof Error ? error.message : "Unknown error";
    res.status(500).json({
      error: `Deployment failed: ${errorMessage}`,
    });
  }
};
