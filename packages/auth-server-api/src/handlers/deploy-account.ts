import type { Request, Response } from "express";
import { type Address, createPublicClient, createWalletClient, type Hex, http } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { waitForTransactionReceipt } from "viem/actions";
import { getGeneralPaymasterInput } from "viem/zksync";
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
  paymaster?: Address;
};

// Deploy account endpoint
export const deployAccountHandler = async (req: Request, res: Response): Promise<void> => {
  try {
    console.log("[DEBUG] deploy-account handler - Raw request body:", JSON.stringify(req.body, null, 2));

    // Validate request body
    const validationResult = deployAccountSchema.safeParse(req.body);
    if (!validationResult.success) {
      console.log("[DEBUG] Validation failed:", validationResult.error.errors);
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

    console.log("Deploying account with ID:", accountId);

    // Send transaction
    let txHash: Hex;
    try {
      const txParams: any = {
        to: transaction.to,
        data: transaction.data,
      };

      // Add paymaster if provided
      if (body.paymaster) {
        txParams.paymaster = body.paymaster;
        txParams.paymasterInput = getGeneralPaymasterInput({ innerInput: "0x" });
      }

      txHash = await walletClient.sendTransaction(txParams);
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
        error: `Transaction failed: ${errorMessage}`,
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
      const errorMessage = error instanceof Error ? error.message : "Unknown error";
      res.status(500).json({
        error: `Transaction failed to be mined: ${errorMessage}`,
      });
      return;
    }

    // Check if transaction was successful
    if (receipt.status === "reverted") {
      res.status(500).json({
        error: "Deployment reverted: Transaction execution failed",
      });
      return;
    }

    // Extract deployed address from logs
    let deployedAddress: Address;
    try {
      deployedAddress = getAccountAddressFromLogs(receipt.logs);
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
