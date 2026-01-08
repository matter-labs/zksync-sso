import { ethers } from "ethers";

// System contract addresses (must match exactly with the working script)
const INTEROP_CENTER_ADDRESS = "0x0000000000000000000000000000000000010010";
const INTEROP_HANDLER_ADDRESS = "0x000000000000000000000000000000000001000d";
const L2_INTEROP_ROOT_STORAGE = "0x0000000000000000000000000000000000010008";
const L2_ASSET_ROUTER_ADDRESS = "0x0000000000000000000000000000000000010003";
const L2_NATIVE_TOKEN_VAULT_ADDRESS = "0x0000000000000000000000000000000000010004";

// Encoding version for bridge data
const NEW_ENCODING_VERSION = "0x01";

// ABIs
const ERC20_ABI = [
  "function name() view returns (string)",
  "function symbol() view returns (string)",
  "function decimals() view returns (uint8)",
  "function balanceOf(address account) view returns (uint256)",
  "function approve(address spender, uint256 value) returns (bool)",
];

const NATIVE_TOKEN_VAULT_ABI = [
  "function assetId(address _tokenAddress) view returns (bytes32)",
  "function ensureTokenIsRegistered(address _nativeToken) returns (bytes32)",
  "function tokenAddress(bytes32 _assetId) view returns (address)",
];

const INTEROP_CENTER_ABI = [
  "function sendBundle(bytes calldata _destinationChainId, tuple(bytes to, bytes data, bytes[] callAttributes)[] calldata _callStarters, bytes[] calldata _bundleAttributes) external payable returns (bytes32)",
];

const INTEROP_HANDLER_ABI = [
  "function executeBundle(bytes memory _bundle, tuple(uint256 chainId, uint256 l1BatchNumber, uint256 l2MessageIndex, tuple(uint16 txNumberInBatch, address sender, bytes data) message, bytes32[] proof) memory _proof) external",
];

const INTEROP_ROOT_STORAGE_ABI = [
  "function interopRoots(uint256 chainId, uint256 batchNumber) view returns (bytes32)",
];

// ---- Helper functions ----

/**
 * Convert chainid to minimal bytes representation
 */
function toChainReference(chainid) {
  if (chainid === 0n) {
    return new Uint8Array([0]);
  }
  const hex = chainid.toString(16);
  const paddedHex = hex.length % 2 === 0 ? hex : "0" + hex;
  return ethers.getBytes("0x" + paddedHex);
}

/**
 * Format ERC-7930 interoperable address for EVM chain without address
 */
function formatEvmV1(chainid) {
  const chainReference = toChainReference(BigInt(chainid));
  return ethers.concat([
    "0x00010000", // version (0x0001) + chainType (0x0000 for EVM)
    ethers.toBeHex(chainReference.length, 1),
    chainReference,
    ethers.toBeHex(0, 1), // address length = 0
  ]);
}

/**
 * Format ERC-7930 interoperable address with just address (no chain reference)
 */
function formatEvmV1AddressOnly(addr) {
  return ethers.concat([
    "0x000100000014", // version (0x0001) + chainType (0x0000) + chainRefLength (0x00) + addrLength (0x14 = 20)
    addr,
  ]);
}

/**
 * Compute asset ID
 */
function computeAssetId(chainId, tokenAddress) {
  return ethers.keccak256(
    ethers.AbiCoder.defaultAbiCoder().encode(
      ["uint256", "address", "address"],
      [chainId, L2_NATIVE_TOKEN_VAULT_ADDRESS, tokenAddress],
    ),
  );
}

/**
 * Encode bridge burn data
 */
function encodeBridgeBurnData(amount, receiver, maybeTokenAddress) {
  return ethers.AbiCoder.defaultAbiCoder().encode(
    ["uint256", "address", "address"],
    [amount, receiver, maybeTokenAddress],
  );
}

/**
 * Encode asset router bridgehub deposit data
 */
function encodeAssetRouterBridgehubDepositData(assetId, transferData) {
  return ethers.concat([
    NEW_ENCODING_VERSION,
    ethers.AbiCoder.defaultAbiCoder().encode(
      ["bytes32", "bytes"],
      [assetId, transferData],
    ),
  ]);
}

/**
 * Build second bridge calldata
 */
function buildSecondBridgeCalldata(assetId, amount, receiver, maybeTokenAddress) {
  const inner = encodeBridgeBurnData(amount, receiver, maybeTokenAddress);
  return encodeAssetRouterBridgehubDepositData(assetId, inner);
}

/**
 * Wait until block is finalized
 */
async function waitUntilBlockFinalized(provider, blockNumber, onProgress) {
  onProgress?.(`Waiting for block ${blockNumber} to be finalized...`);

  const POLL_INTERVAL = 100;
  const DEFAULT_TIMEOUT = 300_000;
  let retries = Math.floor(DEFAULT_TIMEOUT / POLL_INTERVAL);

  while (retries > 0) {
    try {
      const block = await provider.getBlock("finalized");
      const executedBlock = block ? block.number : 0;
      if (executedBlock >= blockNumber) {
        return;
      }
    } catch {
      // Continue
    }
    retries -= 1;
    await new Promise((res) => setTimeout(res, POLL_INTERVAL));
  }

  throw new Error("Block was not finalized in time");
}

/**
 * Wait for L2 to L1 log proof
 */
async function waitForL2ToL1LogProof(provider, blockNumber, txHash, onProgress) {
  await waitUntilBlockFinalized(provider, blockNumber, onProgress);

  onProgress?.("Waiting for log proof...");

  const POLL_INTERVAL = 500;
  const MAX_RETRIES = 240;
  let retries = MAX_RETRIES;

  while (retries > 0) {
    try {
      // ethers providers use send() instead of request()
      const proof = await provider.send("zks_getL2ToL1LogProof", [txHash, 0]);

      if (proof) {
        onProgress?.("✓ Log proof obtained");
        return proof;
      }
    } catch {
      // Continue - proof not ready yet
    }
    retries -= 1;
    await new Promise((res) => setTimeout(res, POLL_INTERVAL));
  }

  throw new Error("Log proof did not become available in time");
}

/**
 * Wait until interop root becomes available
 */
async function waitUntilRootBecomesAvailable(provider, chainId, batchNumber, expectedRoot, onProgress) {
  onProgress?.(`Waiting for interop root on destination chain...`);

  console.log("Waiting for interop root:", {
    chainId: chainId.toString(),
    batchNumber: batchNumber.toString(),
    expectedRoot,
  });

  const contract = new ethers.Contract(
    L2_INTEROP_ROOT_STORAGE,
    INTEROP_ROOT_STORAGE_ABI,
    provider,
  );

  const POLL_INTERVAL = 1000;
  const DEFAULT_TIMEOUT = 300_000;
  let retries = Math.floor(DEFAULT_TIMEOUT / POLL_INTERVAL);
  let lastLogTime = Date.now();

  while (retries > 0) {
    let root;
    try {
      root = await contract.interopRoots(chainId, batchNumber);
    } catch {
      root = null;
    }

    // Log every 10 seconds
    if (Date.now() - lastLogTime > 10000) {
      console.log(`Still waiting... Current root: ${root}, Expected: ${expectedRoot}, Retries left: ${retries}`);
      lastLogTime = Date.now();
    }

    if (root && root !== ethers.ZeroHash) {
      console.log(`Root found: ${root}`);
      if (root.toLowerCase() === expectedRoot.toLowerCase()) {
        onProgress?.("✓ Interop root available");
        return;
      } else {
        throw new Error(`Interop root mismatch: expected ${expectedRoot}, got ${root}`);
      }
    }

    retries -= 1;
    await new Promise((res) => setTimeout(res, POLL_INTERVAL));
  }

  throw new Error("Interop root did not become available in time");
}

/**
 * Get token info (name, symbol, decimals)
 */
export async function getTokenInfo(tokenAddress, provider) {
  const token = new ethers.Contract(tokenAddress, ERC20_ABI, provider);

  const [name, symbol, decimals] = await Promise.all([
    token.name(),
    token.symbol(),
    token.decimals(),
  ]);

  return { name, symbol, decimals };
}

/**
 * Get token balance for an address
 */
export async function getTokenBalance(tokenAddress, ownerAddress, provider) {
  const token = new ethers.Contract(tokenAddress, ERC20_ABI, provider);
  const balance = await token.balanceOf(ownerAddress);
  return balance;
}

/**
 * Get wrapped token address on destination chain
 */
export async function getWrappedTokenAddress(tokenAddress, sourceChainId, provider) {
  const assetId = computeAssetId(sourceChainId, tokenAddress);
  const vault = new ethers.Contract(
    L2_NATIVE_TOKEN_VAULT_ADDRESS,
    NATIVE_TOKEN_VAULT_ABI,
    provider,
  );

  try {
    const wrappedAddress = await vault.tokenAddress(assetId);
    if (wrappedAddress && wrappedAddress !== ethers.ZeroAddress) {
      return wrappedAddress;
    }
  } catch {
    // Token not yet bridged
  }

  return null;
}

/**
 * Transfer ERC20 tokens between chains using interop
 * @param providerA - Source chain provider
 * @param providerB - Destination chain provider
 * @param originalTokenAddress - Original token address (always on the origin chain)
 * @param originalChainId - Chain ID where token originated
 */
export async function transferTokensInterop(
  tokenAddress,
  amount,
  recipientAddress,
  senderPrivateKey,
  providerA,
  providerB,
  originalTokenAddress,
  originalChainId,
  onProgress,
) {
  try {
    onProgress?.("Starting token transfer...");

    // Create wallets - A is source, B is destination
    const walletSource = new ethers.Wallet(senderPrivateKey, providerA);
    const walletDest = new ethers.Wallet(senderPrivateKey, providerB);

    const chainSourceId = (await providerA.getNetwork()).chainId;
    const chainDestId = (await providerB.getNetwork()).chainId;

    // Step 1: Check if token is registered, register if needed
    onProgress?.("Step 1/6: Checking token registration...");
    const vault = new ethers.Contract(
      L2_NATIVE_TOKEN_VAULT_ADDRESS,
      NATIVE_TOKEN_VAULT_ABI,
      walletSource,
    );

    // Use original token info for asset ID to ensure consistency across chains
    const assetId = computeAssetId(originalChainId, originalTokenAddress);

    console.log("Asset ID calculation:", {
      originalChainId: originalChainId.toString(),
      originalTokenAddress,
      tokenAddress,
      assetId,
    });

    try {
      const existingAssetId = await vault.assetId(tokenAddress);
      if (existingAssetId && existingAssetId !== ethers.ZeroHash) {
        onProgress?.("✓ Token already registered");
      } else {
        throw new Error("Not registered");
      }
    } catch {
      onProgress?.("Registering token...");
      const registerTx = await vault.ensureTokenIsRegistered(tokenAddress, {
        gasLimit: 5_000_000n,
        gasPrice: 1_000_000_000n,
      });
      await registerTx.wait();
      onProgress?.("✓ Token registered");
    }

    // Step 2: Approve tokens
    onProgress?.("Step 2/6: Approving tokens...");
    const token = new ethers.Contract(tokenAddress, ERC20_ABI, walletSource);
    const approveTx = await token.approve(L2_NATIVE_TOKEN_VAULT_ADDRESS, amount, {
      gasLimit: 100_000n,
      gasPrice: 1_000_000_000n,
    });
    await approveTx.wait();
    onProgress?.("✓ Tokens approved");

    // Step 3: Build interop bundle
    onProgress?.("Step 3/6: Building interop bundle...");
    const secondBridgeCalldata = buildSecondBridgeCalldata(
      assetId,
      amount,
      recipientAddress,
      ethers.ZeroAddress,
    );

    // Build call attributes with indirectCall
    const indirectCallSelector = ethers.id("indirectCall(uint256)").substring(0, 10);
    const callAttributes = [
      indirectCallSelector + ethers.AbiCoder.defaultAbiCoder().encode(["uint256"], [0n]).substring(2),
    ];

    const calls = [{
      to: formatEvmV1AddressOnly(L2_ASSET_ROUTER_ADDRESS),
      data: secondBridgeCalldata,
      callAttributes: callAttributes,
    }];

    // Build bundle attributes
    const unbundlerAddressSelector = ethers.id("unbundlerAddress(bytes)").substring(0, 10);
    const unbundlerAddressEncoded = ethers.AbiCoder.defaultAbiCoder().encode(
      ["bytes"],
      [formatEvmV1AddressOnly(recipientAddress)],
    );
    const bundleAttributes = [
      unbundlerAddressSelector + unbundlerAddressEncoded.substring(2),
    ];

    const destinationChainId = formatEvmV1(chainDestId);

    // Step 4: Send bundle
    onProgress?.("Step 4/6: Sending interop bundle...");

    console.log("Debug - Full bundle params:", {
      assetId,
      sourceChainId: chainSourceId.toString(),
      destChainId: chainDestId.toString(),
      destinationChainId: ethers.hexlify(destinationChainId),
      tokenAddress,
      amount: amount.toString(),
      calls: calls.map((c) => ({
        to: ethers.hexlify(c.to),
        data: c.data.substring(0, 66) + "...", // First 32 bytes
        dataLength: c.data.length,
        callAttributesLength: c.callAttributes.length,
      })),
      bundleAttributesLength: bundleAttributes.length,
    });

    const interopCenter = new ethers.Contract(
      INTEROP_CENTER_ADDRESS,
      INTEROP_CENTER_ABI,
      walletSource,
    );

    // Log the actual function call before sending
    console.log("Calling sendBundle with:", {
      to: INTEROP_CENTER_ADDRESS,
      from: walletSource.address,
      functionName: "sendBundle",
      argsLength: 3,
    });

    let sendTx;
    try {
      sendTx = await interopCenter.sendBundle(destinationChainId, calls, bundleAttributes, {
        gasLimit: 5_000_000n,
        gasPrice: 1_000_000_000n,
      });

      onProgress?.(`✓ Bundle sent: ${sendTx.hash}`);
    } catch (error) {
      console.error("SendBundle call failed:", error);
      console.error("Error details:", {
        message: error.message,
        code: error.code,
        data: error.data,
      });
      throw new Error(`Failed to send bundle: ${error.message}`);
    }

    const sendReceipt = await sendTx.wait();

    console.log("SendBundle receipt:", {
      status: sendReceipt.status,
      gasUsed: sendReceipt.gasUsed.toString(),
      logsCount: sendReceipt.logs.length,
    });

    // Check if transaction succeeded
    if (sendReceipt.status !== 1) {
      throw new Error(`SendBundle transaction reverted. Tx: ${sendTx.hash}. This usually means InteropCenter rejected the bundle.`);
    }

    onProgress?.(`✓ Mined in block ${sendReceipt.blockNumber}`);

    // Step 5: Wait for proof
    onProgress?.("Step 5/6: Waiting for proof...");
    const logProof = await waitForL2ToL1LogProof(
      providerA,
      sendReceipt.blockNumber,
      sendTx.hash,
      onProgress,
    );

    // Step 6: Wait for interop root
    onProgress?.("Step 6/6: Waiting for interop root...");
    await waitUntilRootBecomesAvailable(
      providerB,
      chainSourceId,
      logProof.batch_number,
      logProof.root,
      onProgress,
    );

    // Extract bundle from receipt
    onProgress?.("Extracting bundle from receipt...");
    const iface = new ethers.Interface([
      "event InteropBundleSent(bytes32 l2l1MsgHash, bytes32 interopBundleHash, tuple(bytes1 version, uint256 sourceChainId, uint256 destinationChainId, bytes32 interopBundleSalt, tuple(bytes1 version, bool shadowAccount, address to, address from, uint256 value, bytes data)[] calls, tuple(bytes executionAddress, bytes unbundlerAddress) bundleAttributes) interopBundle)",
    ]);

    let interopBundle = null;
    for (const log of sendReceipt.logs) {
      try {
        const parsed = iface.parseLog({ topics: log.topics, data: log.data });
        if (parsed && parsed.name === "InteropBundleSent") {
          interopBundle = parsed.args.interopBundle;
          break;
        }
      } catch {
        continue;
      }
    }

    if (!interopBundle) {
      throw new Error("InteropBundleSent event not found in receipt");
    }

    // Encode bundle
    const encodedBundle = ethers.AbiCoder.defaultAbiCoder().encode(
      ["tuple(bytes1 version, uint256 sourceChainId, uint256 destinationChainId, bytes32 interopBundleSalt, tuple(bytes1 version, bool shadowAccount, address to, address from, uint256 value, bytes data)[] calls, tuple(bytes executionAddress, bytes unbundlerAddress) bundleAttributes)"],
      [interopBundle],
    );

    const BUNDLE_IDENTIFIER = "0x01";
    const l2ToL1Message = ethers.concat([BUNDLE_IDENTIFIER, encodedBundle]);

    // Build message inclusion proof
    const messageInclusionProof = {
      chainId: chainSourceId,
      l1BatchNumber: logProof.batch_number,
      l2MessageIndex: logProof.id,
      message: {
        txNumberInBatch: sendReceipt.index,
        sender: INTEROP_CENTER_ADDRESS,
        data: l2ToL1Message,
      },
      proof: logProof.proof,
    };

    // Execute bundle on destination chain
    onProgress?.("Executing bundle on destination chain...");
    const interopHandler = new ethers.Contract(
      INTEROP_HANDLER_ADDRESS,
      INTEROP_HANDLER_ABI,
      walletDest,
    );

    const executeTx = await interopHandler.executeBundle(encodedBundle, messageInclusionProof, {
      gasLimit: 10_000_000n,
      gasPrice: 1_000_000_000n,
    });

    onProgress?.(`✓ Execute tx: ${executeTx.hash}`);
    const executeReceipt = await executeTx.wait();

    if (executeReceipt.status !== 1) {
      throw new Error("Execute bundle transaction failed");
    }

    onProgress?.("✓ Token transfer complete!");

    return {
      sendTxHash: sendTx.hash,
      executeTxHash: executeTx.hash,
      batchNumber: logProof.batch_number,
      messageIndex: logProof.id,
    };
  } catch (error) {
    console.error("Token transfer failed:", error);
    throw error;
  }
}
