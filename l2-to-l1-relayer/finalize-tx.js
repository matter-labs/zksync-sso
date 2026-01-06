import { createViemClient, createViemSdk } from "@matterlabs/zksync-js/viem";
import dotenv from "dotenv";
import { readFileSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";
import { createPublicClient, createWalletClient, decodeAbiParameters, decodeFunctionData, http, keccak256 } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { sepolia } from "viem/chains";

dotenv.config();

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Load ABIs
const L1_INTEROP_HANDLER_ABI = JSON.parse(
  readFileSync(join(__dirname, "../aave-interop-demo/utils/abis/L1InteropHandler.json"), "utf-8"),
).abi;
const L1_BRIDGEHUB_ABI = JSON.parse(
  readFileSync(join(__dirname, "../aave-interop-demo/utils/abis/IL1Bridgehub.json"), "utf-8"),
).abi;
const ERC20_ABI = JSON.parse(
  readFileSync(join(__dirname, "../passkey-wallet-app/abis/IERC20.json"), "utf-8"),
).abi;
const WETH_GATEWAY_ABI = JSON.parse(
  readFileSync(join(__dirname, "../passkey-wallet-app/abis/IWrappedTokenGatewayV3.json"), "utf-8"),
).abi;
const AAVE_POOL_ABI = JSON.parse(
  readFileSync(join(__dirname, "../passkey-wallet-app/abis/IPool.json"), "utf-8"),
).abi;
const L2_INTEROP_CENTER_ABI = JSON.parse(
  readFileSync(join(__dirname, "../passkey-wallet-app/abis/L2InteropCenter.json"), "utf-8"),
).abi;

// Configuration
const config = {
  executorPrivateKey: process.env.EXECUTOR_PRIVATE_KEY,
  l2RpcUrl: process.env.L2_RPC_URL || "https://zksync-os-testnet-alpha.zksync.dev/",
  l1RpcUrl: process.env.L1_RPC_URL || "https://eth-sepolia-testnet.api.pocket.network",
  l1InteropHandler: process.env.L1_INTEROP_HANDLER || "0xB0dD4151fdcCaAC990F473533C15BcF8CE10b1de",
  l2InteropCenter: process.env.L2_INTEROP_CENTER || "0xc64315efbdcD90B71B0687E37ea741DE0E6cEFac",
  zksyncChainId: 8022833,
};

// Transaction to finalize
const TX_HASH = "0xbf50e84239a183f0f5fa791641f6d7de0ab1a9c2240990564d72ea7fd89e3b48";

console.log(`\n🚀 Finalizing L2-to-L1 Message\n`);
console.log(`📍 L2 TX: ${TX_HASH}`);
console.log(`📍 L1 Interop Handler: ${config.l1InteropHandler}\n`);

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
const sdk = createViemSdk(client);

const BASE_TOKEN_ADDRESS = "0x000000000000000000000000000000000000800A";

console.log(`💼 Executor: ${executorAccount.address}\n`);

try {
  // Step 1: Get receipt with L2-to-L1 logs
  console.log(`📜 Step 1: Getting receipt with L2-to-L1 logs...`);
  const receipt = await client.zks.getReceiptWithL2ToL1(TX_HASH);

  if (!receipt || receipt.status !== "0x1") {
    throw new Error("Transaction not found or not successful");
  }

  if (!receipt.l2ToL1Logs || receipt.l2ToL1Logs.length === 0) {
    throw new Error("No L2-to-L1 logs found in transaction");
  }

  console.log(`   ✅ Found ${receipt.l2ToL1Logs.length} L2-to-L1 log(s)\n`);

  const baseTokenKey = BASE_TOKEN_ADDRESS.toLowerCase().slice(2);
  const systemMessenger = "0x0000000000000000000000000000000000008008";

  const baseTokenLogIndex = receipt.l2ToL1Logs.findIndex((entry) => {
    const sender = entry.sender?.toLowerCase();
    const key = entry.key?.toLowerCase();
    return sender === BASE_TOKEN_ADDRESS.toLowerCase() || key?.includes(baseTokenKey);
  });

  receipt.l2ToL1Logs.forEach((entry, index) => {
    const sender = entry.sender?.toLowerCase();
    const isSystem = sender === systemMessenger;
    console.log(
      `   Log[${index}] sender: ${entry.sender} key: ${entry.key}`
      + (isSystem ? " (system messenger)" : ""),
    );
  });

  if (baseTokenLogIndex >= 0) {
    console.log(`🧩 Detected base-token withdrawal log (index ${baseTokenLogIndex}).`);
    console.log(`⛓️  Finalizing L2→L1 ETH withdrawal first...`);
    try {
      const status = await sdk.withdrawals.status(TX_HASH);
      if (status.phase === "UNKNOWN") {
        console.log(`   ⚠️  Withdrawal status unknown for ${TX_HASH}; skipping finalize.`);
      } else if (status.phase === "FINALIZED") {
        console.log(`   ✅ Withdrawal already finalized.`);
      } else {
        await sdk.withdrawals.wait(TX_HASH, { for: "ready" });
        await sdk.withdrawals.tryFinalize(TX_HASH);
        const finalized = await sdk.withdrawals.wait(TX_HASH, { for: "finalized" });
        if (finalized?.transactionHash) {
          console.log(`   ✅ Withdrawal finalized on L1: ${finalized.transactionHash}`);
        } else {
          console.log(`   ✅ Withdrawal finalized.`);
        }
      }
    } catch (withdrawError) {
      console.log(`   ⚠️  Withdrawal finalize failed: ${withdrawError.message}`);
    }
    console.log("");
  } else {
    console.log(`ℹ️  No base-token withdrawal log found; proceeding to bundle finalize.\n`);
  }

  // Step 2: Get L2-to-L1 log proof
  console.log(`🔐 Step 2: Fetching Merkle proof...`);
  const l2InteropCenter = config.l2InteropCenter.toLowerCase();
  const logIndex = receipt.l2ToL1Logs.findIndex((entry) => {
    const sender = entry.sender?.toLowerCase();
    const key = entry.key?.toLowerCase();
    return sender === l2InteropCenter || key?.includes(l2InteropCenter.slice(2));
  });
  const proofIndex = logIndex >= 0 ? logIndex : 0;
  const proof = await client.zks.getL2ToL1LogProof(TX_HASH, proofIndex);

  if (!proof) {
    throw new Error("Could not get L2-to-L1 proof (may not be finalized yet)");
  }

  console.log(`   ✅ Proof obtained`);
  console.log(`   Batch: ${proof.batchNumber}, Index: ${proof.id}\n`);

  // Step 3: Build parameters
  console.log(`📦 Step 3: Building execution parameters...`);

  const log = receipt.l2ToL1Logs[proofIndex];
  let sender = log.sender;
  if (sender?.toLowerCase() === systemMessenger && log.key?.length >= 66) {
    sender = `0x${log.key.slice(-40)}`;
  }
  if (!sender) {
    throw new Error("Missing L2 sender in L2-to-L1 log");
  }

  const candidateLogs = receipt.logs.filter((entry) => entry.data && entry.data.length > 130);
  let messageLog = null;
  let message = null;

  if (log.value) {
    const expectedHash = log.value.toLowerCase();
    for (const entry of candidateLogs) {
      const candidate = `0x${entry.data.slice(130)}`;
      if (keccak256(candidate).toLowerCase() === expectedHash) {
        messageLog = entry;
        message = candidate;
        break;
      }
    }
  }

  if (!messageLog) {
    messageLog
      = candidateLogs.find((entry) => entry.address?.toLowerCase() === l2InteropCenter)
      || candidateLogs[0];
    if (messageLog) {
      message = `0x${messageLog.data.slice(130)}`;
    }
  }

  if (!messageLog) {
    throw new Error("Could not locate L2 message log data");
  }
  if (message === "0x") {
    throw new Error("L2 message data is empty; verify the log selection");
  }

  const params = {
    chainId: config.zksyncChainId,
    l2BatchNumber: proof.batchNumber,
    l2MessageIndex: proof.id,
    l2Sender: sender,
    l2TxNumberInBatch: log.tx_number_in_block,
    merkleProof: proof.proof,
    message,
  };

  try {
    const [l2Caller, ops] = decodeAbiParameters(
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
    console.log(`   L2 Caller: ${l2Caller}`);
    console.log(`   Ops Count: ${ops.length}`);
    if (ops.length > 0) {
      const [firstOp] = ops;
      const selector = firstOp.data?.slice(0, 10);
      console.log(`   Op[0] Target: ${firstOp.target}`);
      console.log(`   Op[0] Value: ${firstOp.value}`);
      console.log(`   Op[0] Selector: ${selector}`);
      try {
        const decoded = decodeFunctionData({
          abi: ERC20_ABI,
          data: firstOp.data,
        });
        if (decoded.functionName === "approve") {
          const [spender, amount] = decoded.args;
          console.log(`   Op[0] Spender: ${spender}`);
          console.log(`   Op[0] Amount: ${amount}`);
        }
      } catch (op0Error) {
        console.log(`   Op[0] decode error: ${op0Error.message}`);
      }
    }
    if (ops.length > 1) {
      const secondOp = ops[1];
      const selector = secondOp.data?.slice(0, 10);
      console.log(`   Op[1] Target: ${secondOp.target}`);
      console.log(`   Op[1] Value: ${secondOp.value}`);
      console.log(`   Op[1] Selector: ${selector}`);
      try {
        const decoded = decodeFunctionData({
          abi: WETH_GATEWAY_ABI,
          data: secondOp.data,
        });
        if (decoded.functionName === "withdrawETH") {
          const [pool, amount, to] = decoded.args;
          console.log(`   Op[1] Pool: ${pool}`);
          console.log(`   Op[1] Amount: ${amount}`);
          console.log(`   Op[1] To: ${to}`);
        }
      } catch (op1Error) {
        console.log(`   Op[1] decode error: ${op1Error.message}`);
      }
    }
    if (ops.length > 2) {
      const thirdOp = ops[2];
      const selector = thirdOp.data?.slice(0, 10);
      console.log(`   Op[2] Target: ${thirdOp.target}`);
      console.log(`   Op[2] Value: ${thirdOp.value}`);
      console.log(`   Op[2] Selector: ${selector}`);
      try {
        const decoded = decodeFunctionData({
          abi: L1_BRIDGEHUB_ABI,
          data: thirdOp.data,
        });
        if (decoded.functionName === "requestL2TransactionDirect") {
          const [req] = decoded.args;
          console.log(`   Op[2] mintValue: ${req.mintValue}`);
          console.log(`   Op[2] l2GasLimit: ${req.l2GasLimit}`);
          console.log(`   Op[2] l2GasPerPubdata: ${req.l2GasPerPubdataByteLimit}`);
          console.log(`   Op[2] l2Value: ${req.l2Value}`);
          try {
            const gasPrice = await l1.getGasPrice();
            const baseCost = await l1.readContract({
              address: req.chainId === 8022833n ? "0xc4FD2580C3487bba18D63f50301020132342fdbD" : req.chainId,
              abi: L1_BRIDGEHUB_ABI,
              functionName: "l2TransactionBaseCost",
              args: [req.chainId, gasPrice, req.l2GasLimit, req.l2GasPerPubdataByteLimit],
            });
            const hasEnough = req.mintValue >= baseCost;
            console.log(`   Op[2] baseCost: ${baseCost}`);
            console.log(`   Op[2] mintValue >= baseCost: ${hasEnough}\n`);
          } catch (baseCostError) {
            console.log(`   Op[2] baseCost check failed: ${baseCostError.message}\n`);
          }
        } else if (decoded.functionName === "requestL2TransactionTwoBridges") {
          const [req] = decoded.args;
          console.log(`   Op[2] mintValue: ${req.mintValue}`);
          console.log(`   Op[2] l2GasLimit: ${req.l2GasLimit}`);
          console.log(`   Op[2] l2GasPerPubdata: ${req.l2GasPerPubdataByteLimit}`);
          console.log(`   Op[2] l2Value: ${req.l2Value}`);
          console.log(`   Op[2] secondBridgeAddress: ${req.secondBridgeAddress}`);
          console.log(`   Op[2] secondBridgeValue: ${req.secondBridgeValue}`);
          try {
            const gasPrice = await l1.getGasPrice();
            const baseCost = await l1.readContract({
              address: "0xc4FD2580C3487bba18D63f50301020132342fdbD",
              abi: L1_BRIDGEHUB_ABI,
              functionName: "l2TransactionBaseCost",
              args: [req.chainId, gasPrice, req.l2GasLimit, req.l2GasPerPubdataByteLimit],
            });
            const hasEnough = req.mintValue >= baseCost;
            console.log(`   Op[2] baseCost: ${baseCost}`);
            console.log(`   Op[2] mintValue >= baseCost: ${hasEnough}\n`);
          } catch (baseCostError) {
            console.log(`   Op[2] baseCost check failed: ${baseCostError.message}\n`);
          }
        }
      } catch (bridgeDecodeError) {
        console.log(`   Op[2] decode error: ${bridgeDecodeError.message}`);
      }
    }
    const shadowAccount = await l2.readContract({
      address: config.l2InteropCenter,
      abi: L2_INTEROP_CENTER_ABI,
      functionName: "l1ShadowAccount",
      args: [l2Caller],
    });
    console.log(`   L1 Shadow Account: ${shadowAccount}\n`);

    for (let i = 0; i < ops.length; i += 1) {
      const op = ops[i];
      try {
        await l1.call({
          account: shadowAccount,
          to: op.target,
          data: op.data,
          value: op.value,
        });
        console.log(`   ✅ Op[${i}] simulate ok`);
      } catch (opError) {
        const message = opError.shortMessage || opError.message;
        console.log(`   ❌ Op[${i}] simulate failed: ${message}`);
        if (opError.data) {
          console.log(`   Op[${i}] revert data: ${opError.data}`);
        }
      }
    }
    console.log("");

    try {
      const aToken = "0x5b071b590a59395fE4025A0Ccc1FcC931AAc1830";
      const [aBalance, aAllowance] = await Promise.all([
        l1.readContract({
          address: aToken,
          abi: ERC20_ABI,
          functionName: "balanceOf",
          args: [shadowAccount],
        }),
        l1.readContract({
          address: aToken,
          abi: ERC20_ABI,
          functionName: "allowance",
          args: [shadowAccount, "0x387d311e47e80b498169e6fb51d3193167d89F7D"],
        }),
      ]);
      console.log(`   aETH Balance: ${aBalance}`);
      console.log(`   aETH Allowance: ${aAllowance}\n`);
      try {
        const wethToken = "0xC558DBdd856501FCd9aaF1E62eae57A9F0629a3c";
        const poolAddress = "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951";
        const reserve = await l1.readContract({
          address: poolAddress,
          abi: AAVE_POOL_ABI,
          functionName: "getReserveData",
          args: [wethToken],
        });
        if (reserve?.aTokenAddress) {
          console.log(`   Pool aTokenAddress: ${reserve.aTokenAddress}\n`);
        } else {
          console.log(`   Pool reserve data loaded\n`);
        }
      } catch (reserveError) {
        console.log(`   ⚠️  Failed to fetch pool reserve data: ${reserveError.message}\n`);
      }
    } catch (tokenError) {
      console.log(`   ⚠️  Failed to fetch aETH balance/allowance: ${tokenError.message}`);
    }
  } catch (decodeError) {
    console.log(`   ⚠️  Failed to decode message or fetch shadow account: ${decodeError.message}`);
  }

  console.log(`   Chain ID: ${params.chainId}`);
  console.log(`   Batch: ${params.l2BatchNumber}`);
  console.log(`   Message Index: ${params.l2MessageIndex}`);
  console.log(`   Sender: ${params.l2Sender}\n`);

  // Step 4: Execute on L1
  console.log(`🔗 Step 4: Executing on L1...`);
  console.log(`   Calling L1InteropHandler.receiveInteropFromL2()...`);

  const gasPrice = await l1.getGasPrice();
  let gasEstimate;
  try {
    gasEstimate = await l1.estimateContractGas({
      address: config.l1InteropHandler,
      abi: L1_INTEROP_HANDLER_ABI,
      functionName: "receiveInteropFromL2",
      args: [params],
      account: executorAccount,
      value: 0n,
      gasPrice,
    });
  } catch (estimateError) {
    console.log(`   ⚠️  Gas estimate failed: ${estimateError.message}`);
    gasEstimate = 2_000_000n;
  }
  const estimatedCost = gasEstimate * gasPrice;
  console.log(`   Gas estimate: ${gasEstimate}`);
  console.log(`   gasPrice: ${gasPrice}`);
  console.log(`   Estimated max tx cost: ${estimatedCost}\n`);

  try {
    await l1.simulateContract({
      address: config.l1InteropHandler,
      abi: L1_INTEROP_HANDLER_ABI,
      functionName: "receiveInteropFromL2",
      args: [params],
      account: executorAccount,
      value: 0n,
      gas: gasEstimate,
      gasPrice,
    });
  } catch (simulateError) {
    console.log(`   ❌ Simulation failed: ${simulateError.shortMessage || simulateError.message}`);
    if (simulateError.data) {
      console.log(`   Revert data: ${simulateError.data}`);
    }
    if (simulateError.cause?.data) {
      console.log(`   Revert cause data: ${simulateError.cause.data}`);
    }
    throw simulateError;
  }

  const hash = await l1Wallet.writeContract({
    address: config.l1InteropHandler,
    abi: L1_INTEROP_HANDLER_ABI,
    functionName: "receiveInteropFromL2",
    args: [params],
    account: executorAccount,
    value: 0n,
    gas: gasEstimate,
    gasPrice,
  });

  console.log(`   ✅ L1 TX submitted: ${hash}`);
  console.log(`   🔗 Etherscan: https://sepolia.etherscan.io/tx/${hash}\n`);

  // Step 5: Wait for confirmation
  console.log(`⏳ Step 5: Waiting for L1 confirmation...`);
  const l1Receipt = await l1.waitForTransactionReceipt({ hash });

  if (l1Receipt.status === "success") {
    console.log(`   ✅ L1 execution confirmed in block ${l1Receipt.blockNumber}`);
    console.log(`\n🎉 Success! L2-to-L1 message executed on L1\n`);
  } else {
    console.log(`   ❌ L1 execution failed`);
    console.log(`   Check Etherscan for details\n`);
  }
} catch (error) {
  console.error(`\n❌ Error:`, error.message);

  if (error.message.includes("not finalized")) {
    console.log(`\n⏳ This transaction may not be finalized yet.`);
    console.log(`   L2-to-L1 messages take ~15 minutes to finalize.`);
    console.log(`   Please wait and try again.\n`);
  }

  process.exit(1);
}
