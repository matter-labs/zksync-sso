/**
 * Interop messaging between two local ZKsync chains
 * Based on the tester.js workflow from zksync-os-workflows
 */

import { createPublicClient, createWalletClient, http, stringToHex } from "viem";
import { privateKeyToAccount } from "viem/accounts";

// Local chain configuration
const CHAIN_A_RPC = "http://localhost:3050";
const CHAIN_B_RPC = "http://localhost:3051";

// Rich wallet from anvil (same as in start-tester.sh)
const DEFAULT_PRIVATE_KEY = "0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110";

// System contract addresses
const L1_MESSENGER_ADDRESS = "0x0000000000000000000000000000000000008008";
const L2_MESSAGE_VERIFICATION_ADDRESS = "0x0000000000000000000000000000000000010009";
const L2_INTEROP_ROOT_STORAGE = "0x0000000000000000000000000000000000010008";

// ABIs
const IL1MessengerAbi = [
  {
    type: "function",
    name: "sendToL1",
    inputs: [{ name: "_message", type: "bytes" }],
    outputs: [{ name: "", type: "bytes32" }],
    stateMutability: "nonpayable",
  },
];

const InteropRootStorageAbi = [
  {
    type: "function",
    name: "interopRoots",
    inputs: [
      { name: "chainId", type: "uint256" },
      { name: "batchNumber", type: "uint256" },
    ],
    outputs: [{ name: "", type: "bytes32" }],
    stateMutability: "view",
  },
];

const MessageVerificationAbi = [
  {
    type: "function",
    name: "proveL2MessageInclusionShared",
    inputs: [
      { name: "chainId", type: "uint256" },
      { name: "batchNumber", type: "uint256" },
      { name: "index", type: "uint256" },
      {
        name: "message",
        type: "tuple",
        components: [
          { name: "txNumberInBatch", type: "uint16" },
          { name: "sender", type: "address" },
          { name: "data", type: "bytes" },
        ],
      },
      { name: "proof", type: "bytes32[]" },
    ],
    outputs: [{ name: "", type: "bool" }],
    stateMutability: "view",
  },
];

/**
 * Wait for a block to be finalized
 */
async function waitUntilBlockFinalized(client, blockNumber, onProgress) {
  onProgress?.("Waiting for block to be finalized...");
  const POLL_INTERVAL = 100;
  const DEFAULT_TIMEOUT = 60_000;
  let retries = Math.floor(DEFAULT_TIMEOUT / POLL_INTERVAL);

  while (retries > 0) {
    try {
      const block = await client.getBlock({ blockTag: "finalized" });
      const executedBlock = block ? Number(block.number) : 0;

      if (executedBlock >= blockNumber) {
        onProgress?.("✓ Block finalized");
        return;
      }
    } catch {
      // Ignore errors, keep retrying
    }

    retries -= 1;
    await new Promise((res) => setTimeout(res, POLL_INTERVAL));
  }

  throw new Error("Block was not finalized in time");
}

/**
 * Wait for L2-to-L1 log proof to be available
 */
async function waitForL2ToL1LogProof(client, blockNumber, txHash, onProgress) {
  await waitUntilBlockFinalized(client, blockNumber, onProgress);

  onProgress?.("Waiting for log proof...");

  // Poll for log proof
  const POLL_INTERVAL = 500;
  const MAX_RETRIES = 240; // Increased to 120 seconds (2 minutes)
  let retries = MAX_RETRIES;
  let lastError = null;

  while (retries > 0) {
    try {
      const proof = await client.request({
        method: "zks_getL2ToL1LogProof",
        params: [txHash, 0],
      });

      // Log progress every 10 retries (every 5 seconds)
      if (retries % 10 === 0) {
        onProgress?.(`Checking log proof... (${retries} retries remaining)`);
      }

      if (proof) {
        onProgress?.("✓ Log proof available");
        return proof;
      }
    } catch (error) {
      lastError = error;
      // Log errors occasionally for debugging
      if (retries % 10 === 0) {
        console.warn("Error getting log proof:", error.message);
      }
    }

    retries -= 1;
    await new Promise((res) => setTimeout(res, POLL_INTERVAL));
  }

  throw new Error(`Log proof did not become available in time. Last error: ${lastError?.message || "none"}`);
}

/**
 * Wait for interop root to become available on destination chain
 */
async function waitUntilRootBecomesAvailable(
  client,
  chainId,
  batchNumber,
  expectedRoot,
  onProgress,
) {
  onProgress?.("Waiting for interop root on destination chain...");

  const POLL_INTERVAL = 500; // Increased from 100ms to 500ms for browser
  const DEFAULT_TIMEOUT = 120_000; // Increased to 2 minutes
  let retries = Math.floor(DEFAULT_TIMEOUT / POLL_INTERVAL);
  let lastError = null;

  while (retries > 0) {
    try {
      const root = await client.readContract({
        address: L2_INTEROP_ROOT_STORAGE,
        abi: InteropRootStorageAbi,
        functionName: "interopRoots",
        args: [chainId, batchNumber],
      });

      // Log progress every 10 retries (every 5 seconds)
      if (retries % 10 === 0) {
        onProgress?.(`Checking interop root... (${retries} retries remaining, root: ${root ? root.slice(0, 10) + "..." : "null"})`);
      }

      if (root && root !== "0x0000000000000000000000000000000000000000000000000000000000000000") {
        if (root.toLowerCase() === expectedRoot.toLowerCase()) {
          onProgress?.("✓ Interop root available on destination");
          return;
        } else {
          throw new Error(`Interop root mismatch: expected ${expectedRoot}, got ${root}`);
        }
      }
    } catch (error) {
      lastError = error;
      // Log errors occasionally for debugging
      if (retries % 10 === 0) {
        console.warn("Error checking interop root:", error.message);
      }
    }

    retries -= 1;
    await new Promise((res) => setTimeout(res, POLL_INTERVAL));
  }

  throw new Error(`Interop root did not become available in time. Last error: ${lastError?.message || "none"}`);
}

/**
 * Main interop flow: send message from Chain A and verify on Chain B
 */
export async function sendInteropMessage(message, privateKey, onProgress, sourceRpc = CHAIN_A_RPC, destRpc = CHAIN_B_RPC) {
  const account = privateKeyToAccount(privateKey || DEFAULT_PRIVATE_KEY);

  // Create clients for both chains - source and destination
  const clientSource = createPublicClient({
    transport: http(sourceRpc),
  });

  const clientDest = createPublicClient({
    transport: http(destRpc),
  });

  const walletSource = createWalletClient({
    account,
    transport: http(sourceRpc),
  });

  onProgress?.("Connected to source and destination chains");

  // Get chain IDs
  const networkSource = await clientSource.getChainId();
  const networkDest = await clientDest.getChainId();

  onProgress?.(`Source Chain ID: ${networkSource}, Destination Chain ID: ${networkDest}`);

  // Step 1: Send L2-to-L1 message on source chain
  onProgress?.("Sending L2→L1 message on source chain...");

  const messageBytes = stringToHex(message);

  // Estimate gas
  const gasEstimate = await clientSource.estimateContractGas({
    address: L1_MESSENGER_ADDRESS,
    abi: IL1MessengerAbi,
    functionName: "sendToL1",
    args: [messageBytes],
    account: account.address,
  });

  const txHash = await walletSource.writeContract({
    address: L1_MESSENGER_ADDRESS,
    abi: IL1MessengerAbi,
    functionName: "sendToL1",
    args: [messageBytes],
    gas: gasEstimate * 2n,
    maxFeePerGas: 1_000_000_000n,
    maxPriorityFeePerGas: 0n,
  });

  onProgress?.(`✓ Transaction sent: ${txHash}`);

  // Wait for transaction
  const receipt = await clientSource.waitForTransactionReceipt({ hash: txHash });
  onProgress?.(`✓ Transaction mined in block ${receipt.blockNumber}`);

  // Step 2: Wait for proof
  const logProof = await waitForL2ToL1LogProof(
    clientSource,
    Number(receipt.blockNumber),
    txHash,
    onProgress,
  );

  // Extract batch number and message index from log proof
  // viem returns: { batch_number, id (message index), root, proof }
  const batchNumber = logProof.batch_number;
  const messageIndex = logProof.id;

  onProgress?.(`✓ Proof obtained - Batch: ${batchNumber}, Index: ${messageIndex}`);
  console.log("Full logProof object:", logProof); // Debug log

  // Step 3: Wait for interop root on destination chain
  await waitUntilRootBecomesAvailable(
    clientDest,
    networkSource,
    batchNumber,
    logProof.root,
    onProgress,
  );

  // Step 4: Verify on destination chain
  onProgress?.("Verifying message on destination chain...");

  const included = await clientDest.readContract({
    address: L2_MESSAGE_VERIFICATION_ADDRESS,
    abi: MessageVerificationAbi,
    functionName: "proveL2MessageInclusionShared",
    args: [
      networkSource,
      batchNumber,
      messageIndex,
      {
        txNumberInBatch: receipt.transactionIndex,
        sender: account.address,
        data: messageBytes,
      },
      logProof.proof,
    ],
  });

  if (!included) {
    throw new Error("Message was NOT included");
  }

  onProgress?.("✓ Message verified on destination chain!");

  return {
    txHash,
    blockNumber: receipt.blockNumber,
    batchNumber: logProof.id,
    messageIndex: logProof.id,
    verified: included,
  };
}
