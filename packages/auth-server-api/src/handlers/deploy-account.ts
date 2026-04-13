import type { Request, Response } from "express";
import type { PrividiumSiweChain } from "prividium/siwe";
import { type Address, createPublicClient, createWalletClient, type Hex, http, parseEther } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { waitForTransactionReceipt } from "viem/actions";
import { getAccountAddressFromLogs, prepareDeploySmartAccount } from "zksync-sso/client";

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

    // Create transport - use SDK transport if enabled, otherwise direct RPC
    const transport = adminSdk
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

    // Check deployer balance (skipped if MIN_DEPLOYER_BALANCE_ETH is 0 or unset)
    const minBalance = parseEther(env.MIN_DEPLOYER_BALANCE_ETH);
    if (minBalance > 0n) {
      const balance = await publicClient.getBalance({ address: deployerAccount.address });
      if (balance < minBalance) {
        console.error(`Deployer balance too low: ${balance.toString()} < ${minBalance.toString()}`);
        res.status(500).json({
          error: "Deployer doesn't have enough balance to cover deployment",
        });
        return;
      }
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
        type: "legacy",
      });
    } catch (error) {
      console.error("Transaction send failed:", error);
      const errorMessage = error instanceof Error ? error.message : "Unknown error";
      if (errorMessage.includes("insufficient funds")) {
        res.status(500).json({
          error: "Insufficient balance to cover deployment",
        });
        return;
      }
      res.status(500).json({
        error: "Internal server error",
      });
      return;
    }

    console.log("Transaction sent:", txHash);

    // Wait for transaction receipt
    let receipt;
    try {
      receipt = await waitForTransactionReceipt(publicClient, {
        hash: txHash,
      });
    } catch (error) {
      console.error("Transaction receipt failed:", error);
      res.status(500).json({
        error: "Internal server error",
      });
      return;
    }

    // Check if transaction was successful
    if (receipt.status === "reverted") {
      res.status(500).json({
        error: "Deployment reverted",
      });
      return;
    }

    // Extract deployed address from logs
    let deployedAddress: Address;
    try {
      deployedAddress = getAccountAddressFromLogs(receipt.logs);
    } catch (error) {
      console.error("Failed to extract address from logs:", error);
      res.status(500).json({
        error: "Internal server error",
      });
      return;
    }

    console.log("Account deployed at:", deployedAddress);

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
          deployedAddress,
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
          [deployedAddress],
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
    res.json({ address: deployedAddress });
  } catch (error) {
    console.error("Deployment error:", error);
    res.status(500).json({
      error: "Internal server error",
    });
  }
};
