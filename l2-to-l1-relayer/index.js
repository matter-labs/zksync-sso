import { createViemClient } from "@matterlabs/zksync-js/viem";
import dotenv from "dotenv";
import { readFileSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";
import { createPublicClient, createWalletClient, http } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { sepolia } from "viem/chains";

dotenv.config();

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Load ABIs
const L1_INTEROP_HANDLER_ABI = JSON.parse(
  readFileSync(join(__dirname, "abis/L1InteropHandler.json"), "utf-8"),
).abi;

// Configuration
const config = {
  executorPrivateKey: process.env.EXECUTOR_PRIVATE_KEY,
  l2RpcUrl: process.env.L2_RPC_URL || "https://zksync-os-testnet-alpha.zksync.dev/",
  l1RpcUrl: process.env.L1_RPC_URL || "https://eth-sepolia.g.alchemy.com/v2/Oa5oz2Y9QWGrxv8_0tqabXz_RFc0tqLU",
  l2InteropCenter: process.env.L2_INTEROP_CENTER || "0xc64315efbdcD90B71B0687E37ea741DE0E6cEFac",
  l1InteropHandler: process.env.L1_INTEROP_HANDLER || "0xB0dD4151fdcCaAC990F473533C15BcF8CE10b1de",
  pollInterval: parseInt(process.env.POLL_INTERVAL || "30000"),
  finalizationWait: parseInt(process.env.FINALIZATION_WAIT || "900000"), // 15 minutes
  zksyncChainId: 8022833,
};

// Initialize clients
const executorAccount = privateKeyToAccount(config.executorPrivateKey);

const l1 = createPublicClient({
  chain: sepolia,
  transport: http(config.l1RpcUrl),
});

const l2 = createPublicClient({
  transport: http(config.l2RpcUrl),
});

const l1Wallet = createWalletClient({
  account: executorAccount,
  chain: sepolia,
  transport: http(config.l1RpcUrl),
});

const l2Wallet = createWalletClient({
  account: executorAccount,
  transport: http(config.l2RpcUrl),
});

// Create ZKSync client
const client = createViemClient({ l1, l2, l1Wallet, l2Wallet });

console.log(`\n🤖 L2-to-L1 Relayer Service Starting...`);
console.log(`📍 Executor Address: ${executorAccount.address}`);
console.log(`📍 L2 Interop Center: ${config.l2InteropCenter}`);
console.log(`📍 L1 Interop Handler: ${config.l1InteropHandler}`);
console.log(`⏱️  Finalization Wait: ${config.finalizationWait / 1000 / 60} minutes\n`);

// In-memory queue for pending messages
const pendingMessages = new Map();
let lastProcessedBlock = 0n;

/**
 * Fetch recent transactions to L2InteropCenter
 */
async function fetchL2Events() {
  try {
    const currentBlock = await l2.getBlockNumber();

    // On first run, start from recent blocks
    if (lastProcessedBlock === 0n) {
      lastProcessedBlock = currentBlock - 10n; // Look back 10 blocks
    }

    if (currentBlock <= lastProcessedBlock) {
      return;
    }

    console.log(`🔍 Scanning L2 blocks ${lastProcessedBlock} to ${currentBlock}...`);

    // Scan blocks for transactions to L2InteropCenter
    for (let blockNum = lastProcessedBlock + 1n; blockNum <= currentBlock; blockNum++) {
      const block = await l2.getBlock({
        blockNumber: blockNum,
        includeTransactions: true,
      });

      if (!block || !block.transactions) continue;

      // Process transactions in batches for better performance
      for (const tx of block.transactions) {
        // Check if transaction calls L2InteropCenter (could be via account.execute)
        // We need to check the receipt for L2-to-L1 logs instead
        try {
          const receipt = await client.zks.getReceiptWithL2ToL1(tx.hash);

          if (!receipt || receipt.status !== "0x1") continue; // 0x1 = success
          if (!receipt.l2ToL1Logs || receipt.l2ToL1Logs.length === 0) continue;

          // Check if the L2-to-L1 log is from our L2InteropCenter
          const l2ToL1Log = receipt.l2ToL1Logs[0];

          // The log sender should be the system contract (0x8008)
          // and the key should match our L2InteropCenter
          if (!l2ToL1Log.key?.toLowerCase().includes(config.l2InteropCenter.slice(2).toLowerCase())) {
            continue;
          }

          // Skip if already processed
          if (pendingMessages.has(tx.hash)) continue;

          console.log(`\n📨 New L2-to-L1 message detected!`);
          console.log(`   Sender: ${tx.from}`);
          console.log(`   L2 TX: ${tx.hash}`);
          console.log(`   Block: ${blockNum}`);
          console.log(`   L2-to-L1 Log: ${l2ToL1Log.value}`);

          // Schedule for execution after finalization period
          const executeAt = Date.now() + config.finalizationWait;

          pendingMessages.set(tx.hash, {
            sender: tx.from,
            txHash: tx.hash,
            blockNumber: blockNum,
            executeAt,
            receipt,
            l2ToL1Log,
            status: "pending",
          });

          console.log(`   ⏳ Scheduled for execution at: ${new Date(executeAt).toLocaleTimeString()}`);
        } catch {
          // Skip transactions that fail to get receipt (likely not relevant)
          continue;
        }
      }
    }

    lastProcessedBlock = currentBlock;
  } catch (error) {
    console.error("❌ Error fetching L2 events:", error.message);
  }
}

/**
 * Execute pending messages that are ready
 */
async function executePendingMessages() {
  const now = Date.now();

  for (const [, message] of pendingMessages.entries()) {
    if (message.status !== "pending") continue;
    if (message.executeAt > now) continue;

    console.log(`\n🚀 Executing L2-to-L1 message on L1...`);
    console.log(`   L2 TX: ${message.txHash}`);

    try {
      message.status = "executing";

      // Get L2-to-L1 log proof
      console.log(`   📜 Fetching proof...`);
      const proof = await client.zks.getL2ToL1LogProof(message.txHash, 0);

      if (!proof) {
        throw new Error("Could not get L2-to-L1 proof (may not be finalized yet)");
      }

      const receipt = message.receipt;
      const log = receipt.l2ToL1Logs[0];
      const sender = receipt.to;

      if (!sender) {
        throw new Error("Missing sender in receipt");
      }

      // Build parameters for receiveInteropFromL2
      const params = {
        chainId: config.zksyncChainId,
        l2BatchNumber: proof.batchNumber,
        l2MessageIndex: proof.id,
        l2Sender: sender,
        l2TxNumberInBatch: log.tx_number_in_block,
        merkleProof: proof.proof,
        message: "0x" + receipt.logs[0].data.slice(130),
      };

      console.log(`   📤 Calling L1InteropHandler.receiveInteropFromL2()...`);
      console.log(`   Batch: ${params.l2BatchNumber}, Index: ${params.l2MessageIndex}`);

      // Call receiveInteropFromL2 on L1
      const hash = await l1Wallet.writeContract({
        address: config.l1InteropHandler,
        abi: L1_INTEROP_HANDLER_ABI,
        functionName: "receiveInteropFromL2",
        args: [params],
        account: executorAccount,
      });

      console.log(`   ✅ L1 Execution TX: ${hash}`);
      console.log(`   🔗 Etherscan: https://sepolia.etherscan.io/tx/${hash}`);

      // Wait for confirmation
      const l1Receipt = await l1.waitForTransactionReceipt({ hash });

      if (l1Receipt.status === "success") {
        console.log(`   ✅ L1 execution confirmed in block ${l1Receipt.blockNumber}`);
        message.status = "executed";
        message.l1TxHash = hash;
        message.l1Receipt = l1Receipt;
      } else {
        console.log(`   ❌ L1 execution failed`);
        message.status = "failed";
        message.error = "L1 transaction reverted";
      }
    } catch (error) {
      console.error(`   ❌ Execution error:`, error.message);
      message.status = "failed";
      message.error = error.message;

      // Retry after 5 minutes if it's a temporary error
      if (error.message.includes("not finalized")
        || error.message.includes("nonce")
        || error.message.includes("gas")) {
        console.log(`   ⏳ Retrying in 5 minutes...`);
        message.executeAt = Date.now() + 300000; // 5 minutes
        message.status = "pending";
      }
    }
  }

  // Clean up old executed/failed messages (keep for 1 hour)
  const oneHourAgo = Date.now() - 3600000;
  for (const [hash, message] of pendingMessages.entries()) {
    if (message.status !== "pending" && message.executeAt < oneHourAgo) {
      pendingMessages.delete(hash);
    }
  }
}

/**
 * Display status
 */
function displayStatus() {
  const pending = Array.from(pendingMessages.values()).filter((m) => m.status === "pending");
  const executing = Array.from(pendingMessages.values()).filter((m) => m.status === "executing");
  const executed = Array.from(pendingMessages.values()).filter((m) => m.status === "executed");
  const failed = Array.from(pendingMessages.values()).filter((m) => m.status === "failed");

  console.log(`\n📊 Status: ${pending.length} pending | ${executing.length} executing | ${executed.length} executed | ${failed.length} failed`);

  if (pending.length > 0) {
    console.log(`\n⏳ Pending messages:`);
    for (const msg of pending) {
      const timeLeft = Math.max(0, Math.ceil((msg.executeAt - Date.now()) / 1000 / 60));
      console.log(`   - ${msg.txHash.slice(0, 10)}... (${timeLeft} min remaining)`);
    }
  }
}

/**
 * Main loop
 */
async function main() {
  console.log(`\n🔄 Starting monitoring loop...\n`);

  setInterval(async () => {
    await fetchL2Events();
    await executePendingMessages();
    displayStatus();
  }, config.pollInterval);

  // Initial fetch
  await fetchL2Events();
  displayStatus();
}

// Handle graceful shutdown
process.on("SIGINT", () => {
  console.log("\n\n👋 Shutting down relayer service...");
  console.log(`📊 Final stats: ${pendingMessages.size} messages in queue`);
  process.exit(0);
});

// Start the service
main().catch((error) => {
  console.error("💥 Fatal error:", error);
  process.exit(1);
});
