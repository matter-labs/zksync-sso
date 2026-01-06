import { createViemClient, createViemSdk } from "@matterlabs/zksync-js/viem";
import dotenv from "dotenv";
import { existsSync, readFileSync, writeFileSync } from "fs";
import { createServer } from "http";
import { dirname, join } from "path";
import { fileURLToPath } from "url";
import { createPublicClient, createWalletClient, decodeAbiParameters, decodeFunctionData, formatEther, http, keccak256 } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { sepolia } from "viem/chains";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Load environment variables
dotenv.config({ path: join(__dirname, ".env") });

// Configuration
const CHECK_INTERVAL = parseInt(process.env.POLL_INTERVAL || "30000", 10);
const EXECUTOR_PRIVATE_KEY = process.env.EXECUTOR_PRIVATE_KEY;

if (!EXECUTOR_PRIVATE_KEY) {
  console.error("❌ EXECUTOR_PRIVATE_KEY not found in .env file");
  process.exit(1);
}
const PENDING_TXS_FILE = join(__dirname, "pending-txs.json");
const FINALIZED_TXS_FILE = join(__dirname, "finalized-txs.json");
const SCAN_STATE_FILE = join(__dirname, "scan-state.json");
const STATUS_PORT = 4340;

// Load ABIs
const L1_INTEROP_HANDLER_ABI = JSON.parse(
  readFileSync(join(__dirname, "abis/L1InteropHandler.json"), "utf-8"),
).abi;
const WETH_GATEWAY_ABI = JSON.parse(
  readFileSync(join(__dirname, "abis/IWrappedTokenGatewayV3.json"), "utf-8"),
).abi;
const AAVE_POOL_ABI = JSON.parse(
  readFileSync(join(__dirname, "abis/IPool.json"), "utf-8"),
).abi;

// RPC URLs
const L1_RPC_URL = process.env.L1_RPC_URL || "https://eth-sepolia.g.alchemy.com/v2/Oa5oz2Y9QWGrxv8_0tqabXz_RFc0tqLU";
const L2_RPC_URL = process.env.L2_RPC_URL || "https://zksync-os-testnet-alpha.zksync.dev/";

// Contract addresses
const L1_INTEROP_HANDLER = process.env.L1_INTEROP_HANDLER || "0xB0dD4151fdcCaAC990F473533C15BcF8CE10b1de";
const L2_INTEROP_CENTER = process.env.L2_INTEROP_CENTER || "0xc64315efbdcD90B71B0687E37ea741DE0E6cEFac";
const BASE_TOKEN_ADDRESS = "0x000000000000000000000000000000000000800A";

// Clients
const executorAccount = privateKeyToAccount(EXECUTOR_PRIVATE_KEY);

const l1 = createPublicClient({
  chain: sepolia,
  transport: http(L1_RPC_URL),
});

const l2 = createPublicClient({
  transport: http(L2_RPC_URL),
});

const l1Wallet = createWalletClient({
  account: executorAccount,
  chain: sepolia,
  transport: http(L1_RPC_URL),
});

const l2Wallet = createWalletClient({
  account: executorAccount,
  transport: http(L2_RPC_URL),
});

// Create ZKSync client
const client = createViemClient({ l1, l2, l1Wallet, l2Wallet });
const sdk = createViemSdk(client);

// Load or initialize pending transactions
function loadPendingTxs() {
  if (existsSync(PENDING_TXS_FILE)) {
    const data = readFileSync(PENDING_TXS_FILE, "utf-8");
    const sanitized = data
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .replace(/^\s*\/\/.*$/gm, "")
      .trim();
    if (!sanitized) {
      return [];
    }
    return JSON.parse(sanitized);
  }
  return [];
}

function savePendingTxs(txs) {
  writeFileSync(PENDING_TXS_FILE, JSON.stringify(txs, null, 2));
}

function loadFinalizedTxs() {
  if (existsSync(FINALIZED_TXS_FILE)) {
    const data = readFileSync(FINALIZED_TXS_FILE, "utf-8");
    return JSON.parse(data);
  }
  return [];
}

function saveFinalizedTxs(txs) {
  writeFileSync(FINALIZED_TXS_FILE, JSON.stringify(txs, null, 2));
}

function loadScanState() {
  if (existsSync(SCAN_STATE_FILE)) {
    const data = readFileSync(SCAN_STATE_FILE, "utf-8");
    const parsed = JSON.parse(data);
    return { lastBlock: BigInt(parsed.lastBlock || 0) };
  }
  return { lastBlock: 0n };
}

function saveScanState(state) {
  writeFileSync(
    SCAN_STATE_FILE,
    JSON.stringify({ lastBlock: state.lastBlock.toString() }, null, 2),
  );
}

async function extractTxMetadata(receipt) {
  try {
    if (!receipt || !receipt.l2ToL1Logs || receipt.l2ToL1Logs.length === 0) {
      return { action: "Unknown", amount: "0" };
    }

    // Find the L2InteropCenter log
    const l2InteropCenter = L2_INTEROP_CENTER.toLowerCase();
    const logIndex = receipt.l2ToL1Logs.findIndex((entry) => {
      const key = entry.key?.toLowerCase();
      const addressFromKey = key ? "0x" + key.slice(-40) : "";
      return addressFromKey === l2InteropCenter;
    });

    if (logIndex === -1) {
      return { action: "Unknown", amount: "0" };
    }

    // Decode the message to get operations
    const log = receipt.l2ToL1Logs[logIndex];
    const message = log.value;

    if (!message || message === "0x" || message.length < 66) {
      // Message too short or invalid
      return { action: "Unknown", amount: "0" };
    }

    let ops;
    try {
      [, ops] = decodeAbiParameters(
        [
          { name: "l2Caller", type: "address" },
          {
            name: "ops",
            type: "tuple[]",
            components: [
              { name: "target", type: "address" },
              { name: "value", type: "uint256" },
              { name: "data", type: "bytes" },
            ],
          },
        ],
        message,
      );
    } catch {
      // Failed to decode message - invalid format
      return { action: "Unknown", amount: "0" };
    }

    if (!ops || ops.length === 0) {
      return { action: "Unknown", amount: "0" };
    }

    // Check first operation to determine action type
    const firstOp = ops[0];
    try {
      // Try to decode as depositETH (WETH Gateway)
      const decoded = decodeFunctionData({
        abi: WETH_GATEWAY_ABI,
        data: firstOp.data,
      });

      if (decoded.functionName === "depositETH") {
        const amount = firstOp.value;
        return {
          action: "Deposit",
          amount: formatEther(amount),
        };
      }
    } catch {
      // Not a deposit, try withdrawal
    }

    try {
      // Try to decode as withdraw (Aave Pool)
      const decoded = decodeFunctionData({
        abi: AAVE_POOL_ABI,
        data: firstOp.data,
      });

      if (decoded.functionName === "withdraw") {
        const [, amount] = decoded.args;
        return {
          action: "Withdrawal",
          amount: formatEther(amount),
        };
      }
    } catch {
      // Not a withdrawal
    }

    return { action: "Unknown", amount: "0" };
  } catch (error) {
    console.error("Error extracting tx metadata:", error.message);
    return { action: "Unknown", amount: "0" };
  }
}

function addPendingTx(hash, metadata = {}) {
  const pending = loadPendingTxs();
  const finalized = loadFinalizedTxs();
  if (pending.some((tx) => tx.hash === hash) || finalized.some((tx) => tx.l2TxHash === hash)) {
    return;
  }
  pending.push({
    hash,
    addedAt: new Date().toISOString(),
    status: "pending",
    action: metadata.action || "Unknown",
    amount: metadata.amount || "0",
  });
  savePendingTxs(pending);
}

async function scanNewBlocks() {
  const state = loadScanState();
  const currentBlock = await l2.getBlockNumber();
  const fallbackStart = currentBlock > 5n ? currentBlock - 5n : 0n;
  const startBlock = state.lastBlock > 0n ? state.lastBlock + 1n : fallbackStart;

  if (startBlock > currentBlock) {
    return;
  }

  const l2InteropCenter = L2_INTEROP_CENTER.toLowerCase();
  for (let blockNumber = startBlock; blockNumber <= currentBlock; blockNumber += 1n) {
    const block = await l2.getBlock({ blockNumber, includeTransactions: true });
    if (!block?.transactions?.length) {
      continue;
    }
    for (const tx of block.transactions) {
      if (!tx || typeof tx === "string") continue;
      if (tx.from?.toLowerCase() !== executorAccount.address.toLowerCase()) continue;
      let receipt;
      try {
        receipt = await client.zks.getReceiptWithL2ToL1(tx.hash);
      } catch {
        continue;
      }
      if (!receipt?.l2ToL1Logs?.length) continue;
      const hasInteropLog = receipt.l2ToL1Logs.some((entry) => {
        const key = entry.key?.toLowerCase();
        const sender = entry.sender?.toLowerCase();
        return sender === l2InteropCenter || key?.includes(l2InteropCenter.slice(2));
      });
      if (!hasInteropLog) continue;
      const metadata = await extractTxMetadata(receipt);
      addPendingTx(tx.hash, metadata);
    }
  }

  saveScanState({ lastBlock: currentBlock });
}

async function finalizeTx(txHash) {
  try {
    console.log(`\n${"=".repeat(80)}`);
    console.log(`🚀 Attempting to finalize: ${txHash}`);
    console.log("=".repeat(80));

    // Step 1: Get receipt with L2-to-L1 logs
    console.log(`📜 Getting receipt with L2-to-L1 logs...`);
    const receipt = await client.zks.getReceiptWithL2ToL1(txHash);

    if (!receipt || receipt.status !== "0x1") {
      console.log("❌ Transaction not found or not successful");
      return { success: false, reason: "tx_not_found" };
    }

    if (!receipt.l2ToL1Logs || receipt.l2ToL1Logs.length === 0) {
      console.log("ℹ️  No L2-to-L1 logs found - nothing to finalize");
      return { success: true, reason: "no_logs" };
    }

    console.log(`✅ Found ${receipt.l2ToL1Logs.length} L2-to-L1 log(s)`);

    const baseTokenKey = BASE_TOKEN_ADDRESS.toLowerCase().slice(2);
    const baseTokenLogIndex = receipt.l2ToL1Logs.findIndex((entry) => {
      const sender = entry.sender?.toLowerCase();
      const key = entry.key?.toLowerCase();
      return sender === BASE_TOKEN_ADDRESS.toLowerCase() || key?.includes(baseTokenKey);
    });

    if (baseTokenLogIndex >= 0) {
      console.log(`💸 Base-token withdrawal log detected (index ${baseTokenLogIndex}).`);
      const status = await sdk.withdrawals.status(txHash);

      if (status.phase === "FINALIZED") {
        console.log(`✅ Withdrawal already finalized for ${txHash}`);
        // Don't return - continue to check L2InteropCenter message
      } else if (status.phase === "READY_TO_FINALIZE") {
        // Ready to finalize - execute now
        console.log(`🚀 Withdrawal is ready - finalizing now...`);
        await sdk.withdrawals.tryFinalize(txHash);
        await sdk.withdrawals.wait(txHash, { for: "finalized" });
        console.log(`✅ Withdrawal finalized for ${txHash}`);
        // Don't return - continue to check L2InteropCenter message
      } else {
        // Not ready yet - return and try again later
        console.log(`⏳ Withdrawal not ready yet (phase: ${status.phase}) - will retry later`);
        return { success: false, reason: "withdrawal_not_ready" };
      }
    }

    // Step 2: Find the log from L2InteropCenter
    // Note: The L2InteropCenter address is in the 'key' field, not 'sender'
    // The sender is the L1 Messenger system contract (0x0000000000000000000000000000000000008008)
    const l2InteropCenter = L2_INTEROP_CENTER.toLowerCase();
    const logIndex = receipt.l2ToL1Logs.findIndex((entry) => {
      // The key contains the L2InteropCenter address (padded to 32 bytes)
      const key = entry.key?.toLowerCase();
      // Remove 0x and get last 40 chars (20 bytes = address)
      const addressFromKey = key ? "0x" + key.slice(-40) : "";
      return addressFromKey === l2InteropCenter;
    });

    if (logIndex === -1) {
      console.log("ℹ️  No logs from L2InteropCenter - nothing to finalize");
      return { success: true, reason: "no_interop_logs" };
    }

    console.log(`📨 Found L2InteropCenter log at index ${logIndex}`);

    // Step 3: Get proof
    console.log(`🔐 Fetching Merkle proof...`);
    let proof;
    try {
      proof = await client.zks.getL2ToL1LogProof(txHash, logIndex);
    } catch (proofError) {
      // Check if it's a "batch not executed yet" error
      if (proofError.message?.includes("not been executed yet")
        || proofError.message?.includes("proof not available")) {
        console.log("⏳ Proof not available yet - L1 batch not executed");
        return { success: false, reason: "proof_not_ready" };
      }
      throw proofError; // Re-throw if it's a different error
    }

    if (!proof) {
      console.log("⏳ Proof not available yet - message not finalized on L2");
      return { success: false, reason: "proof_not_ready" };
    }

    console.log(`✅ Proof obtained`);
    console.log(`   Batch: ${proof.batchNumber}, Message Index: ${proof.id}`);

    // Step 4: Extract message data from logs
    const log = receipt.l2ToL1Logs[logIndex];
    const l2BatchNumber = proof.batchNumber;
    const messageIndex = proof.id;

    console.log(`📋 Building execution parameters...`);
    console.log(`   Batch: ${l2BatchNumber}`);
    console.log(`   Message Index: ${messageIndex}`);

    // Extract sender from log (L2InteropCenter address is in the key field)
    const systemMessenger = "0x0000000000000000000000000000000000008008";
    let sender = log.sender;
    if (sender?.toLowerCase() === systemMessenger.toLowerCase() && log.key?.length >= 66) {
      sender = `0x${log.key.slice(-40)}`;
    }

    // Find the message data in the logs
    const candidateLogs = receipt.logs.filter((entry) => entry.data && entry.data.length > 130);
    let message = null;

    if (log.value) {
      const expectedHash = log.value.toLowerCase();
      for (const entry of candidateLogs) {
        const candidate = `0x${entry.data.slice(130)}`;
        if (keccak256(candidate).toLowerCase() === expectedHash) {
          message = candidate;
          break;
        }
      }
    }

    if (!message) {
      const messageLog
        = candidateLogs.find((entry) => entry.address?.toLowerCase() === L2_INTEROP_CENTER.toLowerCase())
        || candidateLogs[0];
      if (messageLog) {
        message = `0x${messageLog.data.slice(130)}`;
      }
    }

    if (!message || message === "0x") {
      console.log(`❌ Could not extract message data`);
      return { success: false, reason: "no_message" };
    }

    // Build params for receiveInteropFromL2
    const params = {
      chainId: 8022833, // ZKSync OS Testnet chain ID
      l2BatchNumber: l2BatchNumber,
      l2MessageIndex: messageIndex,
      l2Sender: sender,
      l2TxNumberInBatch: log.tx_number_in_block,
      merkleProof: proof.proof,
      message,
    };

    console.log(`   Sender: ${params.l2Sender}`);
    console.log(`   Message length: ${message.length} bytes`);

    // Step 5: Execute on L1
    console.log(`💰 Sending finalization transaction...`);
    const baseGasPrice = await l1.getGasPrice();
    const bumpedGasPrice = (baseGasPrice * 12n) / 10n;
    console.log(`⛽ Gas price: ${baseGasPrice} → ${bumpedGasPrice}`);

    let finalizeHash;
    try {
      finalizeHash = await l1Wallet.writeContract({
        address: L1_INTEROP_HANDLER,
        abi: L1_INTEROP_HANDLER_ABI,
        functionName: "receiveInteropFromL2",
        args: [params],
        gasPrice: bumpedGasPrice,
      });
    } catch (writeError) {
      // Check if error indicates message already finalized
      if (writeError.message?.includes("L1ShadowAccount: call failed")
        || writeError.message?.includes("already finalized")) {
        console.log(`✅ Message appears to be already finalized`);
        return { success: true, reason: "already_finalized" };
      }
      throw writeError;
    }

    console.log(`✅ Transaction sent: ${finalizeHash}`);
    console.log(`⏳ Waiting for confirmation...`);

    let finalizeReceipt;
    try {
      finalizeReceipt = await l1.waitForTransactionReceipt({
        hash: finalizeHash,
        timeout: 300_000,
      });
    } catch (waitError) {
      if (waitError.message?.includes("Timed out")) {
        console.log(`⏳ L1 finalize tx still pending: ${finalizeHash}`);
        return { success: false, reason: "l1_pending", txHash: finalizeHash };
      }
      throw waitError;
    }

    if (finalizeReceipt.status === "success") {
      console.log(`✅ Message finalized successfully!`);
      console.log(`   Block: ${finalizeReceipt.blockNumber}`);
      console.log(`   Gas used: ${finalizeReceipt.gasUsed}`);
      return { success: true, reason: "finalized", txHash: finalizeHash };
    } else {
      console.log(`❌ Finalization transaction failed`);
      return { success: false, reason: "tx_failed" };
    }
  } catch (error) {
    console.error(`❌ Error: ${error.message}`);
    return { success: false, reason: "error", error: error.message };
  }
}

async function processQueue() {
  console.log(`\n${"=".repeat(80)}`);
  console.log(`🔄 Processing queue... ${new Date().toLocaleTimeString()}`);
  console.log("=".repeat(80));

  await scanNewBlocks();

  const pendingTxs = loadPendingTxs();
  const finalizedTxs = loadFinalizedTxs();

  if (pendingTxs.length === 0) {
    console.log("ℹ️  No pending transactions to process");
    return;
  }

  console.log(`📋 Found ${pendingTxs.length} pending transaction(s)`);

  const stillPending = [];

  for (const tx of pendingTxs) {
    // Backfill metadata for old transactions that don't have it
    if (!tx.action || !tx.amount) {
      try {
        const receipt = await client.zks.getReceiptWithL2ToL1(tx.hash);
        if (receipt) {
          const metadata = await extractTxMetadata(receipt);
          tx.action = metadata.action;
          tx.amount = metadata.amount;
          console.log(`📝 Backfilled metadata for ${tx.hash}: ${metadata.action} ${metadata.amount} ETH`);
        }
      } catch {
        // If we can't get metadata, set defaults
        tx.action = tx.action || "Unknown";
        tx.amount = tx.amount || "0";
      }
    }

    const result = await finalizeTx(tx.hash);

    if (result.success) {
      console.log(`✅ Removed from queue: ${tx.hash}`);
      finalizedTxs.unshift({
        l2TxHash: tx.hash,
        l1FinalizeTxHash: result.txHash,
        finalizedAt: new Date().toISOString(),
        action: tx.action,
        amount: tx.amount,
      });
    } else if (result.reason === "proof_not_ready" || result.reason === "l1_pending" || result.reason === "withdrawal_not_ready") {
      console.log(`⏳ Still pending: ${tx.hash}`);
      stillPending.push({
        ...tx,
        lastFinalizeHash: result.txHash || tx.lastFinalizeHash,
        updatedAt: new Date().toISOString(),
      });
    } else {
      console.log(`❌ Failed permanently: ${tx.hash} (${result.reason})`);
    }

    // Small delay between transactions
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }

  savePendingTxs(stillPending);
  saveFinalizedTxs(finalizedTxs.slice(0, 50));
  console.log(`\n📊 Queue updated: ${stillPending.length} remaining`);
}

function startStatusServer() {
  const server = createServer((req, res) => {
    res.setHeader("Access-Control-Allow-Origin", "http://localhost:3000");
    res.setHeader("Access-Control-Allow-Methods", "GET, OPTIONS");
    res.setHeader("Access-Control-Allow-Headers", "Content-Type");
    if (req.method === "OPTIONS") {
      res.writeHead(204);
      res.end();
      return;
    }
    if (req.url === "/status") {
      const pending = loadPendingTxs();
      const finalized = loadFinalizedTxs();
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ pending, finalized }));
      return;
    }
    res.writeHead(404);
    res.end();
  });

  server.listen(STATUS_PORT, () => {
    console.log(`📡 Status server listening on http://localhost:${STATUS_PORT}`);
  });
}

// Main
console.log("🚀 Auto-Finalize Daemon Starting...");
console.log("=".repeat(80));
console.log(`📡 L2 RPC: https://zksync-os-testnet-alpha.zksync.dev/`);
console.log(`📡 L1 RPC: Alchemy Sepolia`);
console.log(`👤 Executor: ${executorAccount.address}`);
console.log(`⏰ Check interval: ${CHECK_INTERVAL / 1000} seconds`);
console.log(`📁 Queue file: ${PENDING_TXS_FILE}`);
console.log(`📁 Finalized file: ${FINALIZED_TXS_FILE}`);
console.log(`📡 Status endpoint: http://localhost:${STATUS_PORT}/status`);
console.log("=".repeat(80));

startStatusServer();

// Run immediately
await processQueue();

// Then run periodically
setInterval(processQueue, CHECK_INTERVAL);

console.log("\n✅ Daemon running. Press Ctrl+C to stop.");
console.log("💡 Add transactions to the queue by editing pending-txs.json");
