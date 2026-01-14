import {
  concat,
  createPublicClient,
  createWalletClient,
  defineChain,
  encodeAbiParameters,
  encodeFunctionData,
  formatEther,
  http,
  keccak256,
  pad,
  parseAbiParameters,
  parseEther,
  parseEventLogs,
  toBytes,
  toHex,
} from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { waitForTransactionReceipt, writeContract } from "viem/actions";
import { registerNewPasskey } from "zksync-sso/client/passkey";
import { base64UrlToUint8Array, unwrapEC2Signature } from "zksync-sso/utils";

import {
  AAVE_CONTRACTS,
  createAaveDepositBundle,
  createAaveWithdrawBundle,
  getAaveBalance,
  getShadowAccount,
} from "./aave-utils.js";
import { sendInteropMessage } from "./interop.js";
import {
  getTokenBalance,
  getTokenInfo,
  getWrappedTokenAddress,
  transferTokensInterop,
} from "./token-interop.js";
import { ENGLISH, initTranslation, PORTUGUESE_BR, SPANISH, updateTranslation } from "./translation.js";

// ZKsync OS configuration
const zksyncOsTestnet = defineChain({
  id: 8022833,
  name: "ZKsync OS Developer Preview",
  nativeCurrency: {
    name: "Ether",
    symbol: "ETH",
    decimals: 18,
  },
  rpcUrls: {
    default: {
      http: ["https://zksync-os-testnet-alpha.zksync.dev/"],
      webSocket: ["wss://zksync-os-testnet-alpha.zksync.dev/ws"],
    },
    public: {
      http: ["https://zksync-os-testnet-alpha.zksync.dev/"],
      webSocket: ["wss://zksync-os-testnet-alpha.zksync.dev/ws"],
    },
  },
  blockExplorers: {
    default: {
      name: "ZKsync OS Explorer",
      url: "https://zksync-os-testnet-alpha.staging-scan-v2.zksync.dev",
    },
  },
});

const DEFAULT_ZKSYNC_OS_RPC_URL = "https://zksync-os-testnet-alpha.zksync.dev/";
const LOCAL_RPC_PROXY_URL = "http://localhost:4339";
// const isBrowser = typeof window !== "undefined";
const shouldUseLocalRpcProxy = false; // Disabled - use direct RPC URL
const ZKSYNC_OS_RPC_URL = shouldUseLocalRpcProxy ? LOCAL_RPC_PROXY_URL : DEFAULT_ZKSYNC_OS_RPC_URL;

// Deployer private key (for deploying accounts and faucet)
// Can be set via VITE_DEPLOYER_PRIVATE_KEY environment variable
const DEPLOYER_PRIVATE_KEY = import.meta.env?.VITE_DEPLOYER_PRIVATE_KEY;

// Interop private key (for local chain interop transactions)
// Can be set via VITE_INTEROP_PRIVATE_KEY environment variable
const INTEROP_PRIVATE_KEY = import.meta.env?.VITE_INTEROP_PRIVATE_KEY || "0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110";

// Contract addresses on ZKsync OS testnet
const CONTRACTS = {
  p256Verifier: "0xD65900405073D912215bC8dEb811dFFD72263065", // P256VerifierNoPrecompile - Pure Solidity!
  passkey: "0xc66A20c63606f221D4d5A39147E3bf3635DD7a39", // WebAuthnValidator (uses pure Solidity verifier)
  session: "0xbd2608f3512A3a163394fbAB99a286E151Bf30be", // SessionKeyValidator
  accountFactory: "0xF52708DE29453BBfd27AA8fC7b4bc7EF87E05892", // MSAFactory
  entryPoint: "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108",
  // bundlerUrl: "https://bundler-api.stage-sso.zksync.dev",
  bundlerUrl: "http://localhost:4337", // Local bundler for better debugging
  oidcKeyRegistry: "0x0000000000000000000000000000000000000000",
};

const STATUS_ENDPOINT = "http://localhost:4340/status";
const L2_EXPLORER_BASE = "https://zksync-os-testnet-alpha.staging-scan-v2.zksync.dev/tx/";
const L1_EXPLORER_BASE = "https://sepolia.etherscan.io/tx/";
// State
let accountAddress = null;
let passkeyCredentials = null;
let publicClient = null;
let sepoliaClient = null;
let shadowAccount = null;
let balanceRefreshInterval = null;
let aaveBalanceRefreshInterval = null;
let activeLanguage = "en";

// LocalStorage keys
const STORAGE_KEY_PASSKEY = "zksync_sso_passkey";
const STORAGE_KEY_ACCOUNT = "zksync_sso_account";
const STORAGE_KEY_TX_METADATA = "zksync_sso_tx_metadata";
const STORAGE_KEY_LANGUAGE = "zksync-interop-demo-language";

// Auto-refresh interval (in milliseconds)
const BALANCE_REFRESH_INTERVAL = 5000; // 5 seconds
const AAVE_BALANCE_REFRESH_INTERVAL = 10000; // 10 seconds (slower to reduce RPC calls)

// Sepolia configuration for L1 operations
const sepolia = defineChain({
  id: 11155111,
  name: "Sepolia",
  nativeCurrency: {
    name: "Sepolia Ether",
    symbol: "ETH",
    decimals: 18,
  },
  rpcUrls: {
    default: {
      http: ["https://eth-sepolia.g.alchemy.com/v2/Oa5oz2Y9QWGrxv8_0tqabXz_RFc0tqLU"],
    },
    public: {
      http: ["https://eth-sepolia.g.alchemy.com/v2/Oa5oz2Y9QWGrxv8_0tqabXz_RFc0tqLU"],
    },
  },
  blockExplorers: {
    default: {
      name: "Etherscan",
      url: "https://sepolia.etherscan.io",
    },
  },
});

const translatedText = () => {
  switch (activeLanguage) {
    case "es":
      return SPANISH.dynamic;
    case "pt":
      return PORTUGUESE_BR.dynamic;
    default:
      return ENGLISH.dynamic;
  }
};

// Initialize
document.addEventListener("DOMContentLoaded", async () => {
  // setup translation
  await setupTranslation();

  // Setup public client for balance checks FIRST
  publicClient = createPublicClient({
    chain: zksyncOsTestnet,
    transport: http(ZKSYNC_OS_RPC_URL),
  });

  // Setup Sepolia client for L1 balance checks
  sepoliaClient = createPublicClient({
    chain: sepolia,
    transport: http("https://eth-sepolia.g.alchemy.com/v2/Oa5oz2Y9QWGrxv8_0tqabXz_RFc0tqLU"),
  });

  // Set network name in header
  const networkNameEl = document.getElementById("headerNetwork");
  if (networkNameEl) {
    networkNameEl.textContent = zksyncOsTestnet.name;
  }

  setupEventListeners();
  loadExistingPasskey();
});

async function setupTranslation() {
  const select = document.getElementById("lang");
  select.addEventListener("change", async () => {
    activeLanguage = select.value;
    localStorage.setItem(STORAGE_KEY_LANGUAGE, select.value);
    await updateTranslation(select.value);
  });
  const savedLanguage = localStorage.getItem(STORAGE_KEY_LANGUAGE);
  if (savedLanguage) {
    activeLanguage = savedLanguage;
    select.value = savedLanguage;
  }
  await initTranslation(activeLanguage);
}

function setupEventListeners() {
  document.getElementById("createPasskeyBtn").addEventListener("click", handleCreatePasskey);
  document.getElementById("deployAccountBtn").addEventListener("click", handleDeployAccount);
  document.getElementById("refreshBalanceBtn").addEventListener("click", handleRefreshBalance);
  document.getElementById("faucetBtn").addEventListener("click", handleFaucet);
  document.getElementById("transferBtn").addEventListener("click", handleTransfer);
  document.getElementById("sendMaxBtn").addEventListener("click", handleSendMax);
  document.getElementById("resetPasskeyBtn").addEventListener("click", handleResetPasskey);
  document.getElementById("resetPasskeyMainBtn").addEventListener("click", handleResetPasskey);
  document.getElementById("aaveDepositBtn").addEventListener("click", handleAaveDeposit);
  document.getElementById("aaveWithdrawBtn").addEventListener("click", handleAaveWithdraw);
  document.getElementById("refreshAaveBalanceBtn").addEventListener("click", refreshAaveBalance);
  document.getElementById("interopSendBtn").addEventListener("click", handleInteropSend);
  document.getElementById("refreshTokenBalancesBtn").addEventListener("click", handleRefreshTokenBalances);
  document.getElementById("tokenTransferBtn").addEventListener("click", handleTokenTransfer);
}

let activityInterval = null;

// Transaction metadata storage helpers
function saveTxMetadata(txHash, action, amount) {
  try {
    const metadata = JSON.parse(localStorage.getItem(STORAGE_KEY_TX_METADATA) || "{}");
    metadata[txHash] = {
      action,
      amount,
      submittedAt: new Date().toISOString(),
    };
    localStorage.setItem(STORAGE_KEY_TX_METADATA, JSON.stringify(metadata));
  } catch (error) {
    console.error("Failed to save tx metadata:", error);
  }
}

function getTxMetadata(txHash) {
  try {
    const metadata = JSON.parse(localStorage.getItem(STORAGE_KEY_TX_METADATA) || "{}");
    return metadata[txHash] || null;
  } catch (error) {
    console.error("Failed to get tx metadata:", error);
    return null;
  }
}

function renderActivity(status) {
  const activityList = document.getElementById("activity-list");
  const activityEmpty = document.getElementById("activity-empty");
  if (!activityList || !activityEmpty) return;

  activityList.textContent = "";

  const pending = status?.pending || [];
  const finalized = status?.finalized || [];

  // Load localStorage metadata to enrich timestamps
  let localStorageMetadata = {};
  try {
    localStorageMetadata = JSON.parse(localStorage.getItem(STORAGE_KEY_TX_METADATA) || "{}");
  } catch (error) {
    console.error("Failed to load localStorage metadata:", error);
  }

  // Only show transactions that belong to the current user (have metadata in localStorage)
  const items = [
    ...pending.map((tx) => {
      const txHash = tx.hash;
      const localMeta = localStorageMetadata[txHash];
      // Only include if we have metadata for this transaction
      if (!localMeta) return null;
      return {
        type: "pending",
        ...tx,
        // Use localStorage submittedAt if available for more accurate sorting
        sortTimestamp: localMeta?.submittedAt || tx.addedAt,
      };
    }).filter(Boolean), // Remove null entries
    ...finalized.map((tx) => {
      const txHash = tx.l2TxHash;
      const localMeta = localStorageMetadata[txHash];
      // Only include if we have metadata for this transaction
      if (!localMeta) return null;
      return {
        type: "finalized",
        ...tx,
        // Use localStorage submittedAt if available, otherwise use finalizedAt
        sortTimestamp: localMeta?.submittedAt || tx.finalizedAt,
      };
    }).filter(Boolean), // Remove null entries
  ];

  // Add transactions from localStorage that aren't in the daemon's status yet
  const daemonTxHashes = new Set([
    ...pending.map((tx) => tx.hash),
    ...finalized.map((tx) => tx.l2TxHash),
  ]);

  for (const [txHash, txMeta] of Object.entries(localStorageMetadata)) {
    if (!daemonTxHashes.has(txHash) && txMeta.action && txMeta.amount) {
      items.push({
        type: "pending",
        hash: txHash,
        action: txMeta.action,
        amount: txMeta.amount,
        sortTimestamp: txMeta.submittedAt || new Date().toISOString(),
      });
    }
  }

  // Sort: Pending first, then by timestamp (newest first)
  items.sort((a, b) => {
    // Primary: Pending transactions always come before finalized
    if (a.type === "pending" && b.type === "finalized") return -1;
    if (a.type === "finalized" && b.type === "pending") return 1;

    // Secondary: Within same status, sort by timestamp (newest first)
    const timeA = new Date(a.sortTimestamp || 0).getTime();
    const timeB = new Date(b.sortTimestamp || 0).getTime();

    if (timeB !== timeA) {
      return timeB - timeA; // Descending order (newest first)
    }

    // Tertiary: if timestamps are equal, use transaction hash as tiebreaker
    const hashA = a.hash || a.l2TxHash || "";
    const hashB = b.hash || b.l2TxHash || "";
    return hashB.localeCompare(hashA);
  });

  if (items.length === 0) {
    activityEmpty.classList.remove("hidden");
    activityList.classList.add("hidden");
    return;
  }

  activityEmpty.classList.add("hidden");
  activityList.classList.remove("hidden");

  items.slice(0, 20).forEach((tx) => {
    const row = document.createElement("div");
    row.className = "info-row";

    const label = document.createElement("span");
    label.className = "info-label";

    // Get metadata from localStorage
    const txHash = tx.hash || tx.l2TxHash;
    const storedMetadata = getTxMetadata(txHash);

    // Use stored metadata if available, fallback to tx data
    let action = storedMetadata?.action || tx.action || "Unknown";
    let amount = storedMetadata?.amount || tx.amount || "0";
    const statusText = tx.type === "pending" ? text.pending : text.finalized;

    if (action !== "Unknown" && amount !== "0") {
      label.textContent = `${amount} ETH (${statusText})`;
    } else {
      label.textContent = statusText;
    }

    const value = document.createElement("span");
    value.className = "info-value";

    // Helper to truncate hash: 0x1234...5678
    const truncateHash = (hash) => {
      if (!hash || hash.length < 12) return hash;
      return `${hash.slice(0, 6)}...${hash.slice(-4)}`;
    };

    // Determine L2 link text based on action type
    const l2Link = document.createElement("a");
    l2Link.href = `${L2_EXPLORER_BASE}${txHash}`;
    l2Link.target = "_blank";
    l2Link.rel = "noreferrer";

    const text = translatedText();

    const l2TxLabel = action === "Deposit"
      ? text.l2DepsitTx
      : action === "Withdrawal"
        ? text.l2WithdrawTx
        : "L2 Tx";
    l2Link.textContent = `${l2TxLabel} ${truncateHash(txHash)}`;

    value.appendChild(l2Link);

    if (tx.type === "finalized" && tx.l1FinalizeTxHash) {
      const sep = document.createTextNode(" \u2022 ");
      value.appendChild(sep);
      const l1Link = document.createElement("a");
      l1Link.href = `${L1_EXPLORER_BASE}${tx.l1FinalizeTxHash}`;
      l1Link.target = "_blank";
      l1Link.rel = "noreferrer";
      l1Link.textContent = `${text.l1FinalizationTx} ${truncateHash(tx.l1FinalizeTxHash)}`;
      value.appendChild(l1Link);
    }

    row.appendChild(label);
    row.appendChild(value);
    activityList.appendChild(row);
  });
}

async function refreshActivity() {
  try {
    const response = await fetch(STATUS_ENDPOINT);
    if (!response.ok) return;
    const status = await response.json();
    renderActivity(status);
  } catch (error) {
    console.log("Activity refresh failed:", error.message);
  }
}

function startActivityPoll() {
  if (activityInterval) clearInterval(activityInterval);
  refreshActivity();
  activityInterval = setInterval(refreshActivity, 15000);
}

// Load existing passkey from localStorage
function loadExistingPasskey() {
  const savedPasskey = localStorage.getItem(STORAGE_KEY_PASSKEY);
  const savedAccount = localStorage.getItem(STORAGE_KEY_ACCOUNT);

  console.log("🔍 Checking for saved passkey...");
  console.log("STORAGE_KEY_PASSKEY:", STORAGE_KEY_PASSKEY);
  console.log("STORAGE_KEY_ACCOUNT:", STORAGE_KEY_ACCOUNT);
  console.log("savedPasskey:", savedPasskey ? "Found" : "Not found");
  console.log("savedAccount:", savedAccount ? "Found" : "Not found");

  if (savedPasskey) {
    passkeyCredentials = JSON.parse(savedPasskey);

    // Show passkey success
    document.getElementById("passkey-input").classList.add("hidden");
    document.getElementById("passkey-success").classList.remove("hidden");
    document.getElementById("credentialIdDisplay").textContent = passkeyCredentials.credentialId;

    // Update PayPal UI step 1
    const passkeyStep = document.getElementById("passkey-step");
    const passkeyIcon = document.getElementById("passkey-icon");
    if (passkeyStep) passkeyStep.classList.add("completed");
    if (passkeyIcon) {
      passkeyIcon.classList.add("completed");
      passkeyIcon.textContent = "✓";
    }

    // Enable deployment button
    document.getElementById("deployAccountBtn").disabled = false;

    console.log("✅ Loaded existing passkey from storage");

    // If account is also deployed, show that too
    if (savedAccount) {
      accountAddress = savedAccount;

      // Show account if already deployed
      document.getElementById("deploy-button-container").classList.add("hidden");
      document.getElementById("deploy-success").classList.remove("hidden");
      document.getElementById("accountAddressDisplay").textContent = accountAddress;

      // Update PayPal UI step 2
      const deployStep = document.getElementById("deploy-step");
      const deployIcon = document.getElementById("deploy-icon");
      if (deployStep) deployStep.classList.add("completed");
      if (deployIcon) {
        deployIcon.classList.add("completed");
        deployIcon.textContent = "✓";
      }

      // Show wallet view instead of setup section
      const setupSection = document.getElementById("setup-section");
      const walletView = document.getElementById("wallet-view");
      if (setupSection) setupSection.classList.add("hidden");
      if (walletView) walletView.classList.remove("hidden");

      // Update wallet address display
      const walletAddressDisplay = document.getElementById("walletAddressDisplay");
      if (walletAddressDisplay) walletAddressDisplay.textContent = accountAddress;

      // Enable transfer
      document.getElementById("transferBtn").disabled = false;
      document.getElementById("sendMaxBtn").disabled = false;

      // Enable Aave buttons
      const aaveDepositBtn = document.getElementById("aaveDepositBtn");
      const aaveWithdrawBtn = document.getElementById("aaveWithdrawBtn");
      if (aaveDepositBtn) aaveDepositBtn.disabled = false;
      if (aaveWithdrawBtn) aaveWithdrawBtn.disabled = false;

      startBalanceAutoRefresh();
      initializeAaveSection();
      startActivityPoll();

      console.log("✅ Loaded existing account from storage");
    }
  }
}

// Save passkey to localStorage
function savePasskeyData() {
  if (passkeyCredentials) {
    localStorage.setItem(STORAGE_KEY_PASSKEY, JSON.stringify(passkeyCredentials));
  }
  if (accountAddress) {
    localStorage.setItem(STORAGE_KEY_ACCOUNT, accountAddress);
  }
}

// Reset passkey
function handleResetPasskey() {
  if (confirm("Are you sure you want to reset your passkey? You will need to create a new one and deploy a new account.")) {
    localStorage.removeItem(STORAGE_KEY_PASSKEY);
    localStorage.removeItem(STORAGE_KEY_ACCOUNT);
    localStorage.removeItem(STORAGE_KEY_TX_METADATA); // Clear transaction history
    location.reload();
  }
}

// Step 1: Create Passkey
async function handleCreatePasskey() {
  const userName = document.getElementById("userName").value.trim();
  const text = translatedText();
  if (!userName) {
    alert(text.enterUserName);
    return;
  }

  const btn = document.getElementById("createPasskeyBtn");
  btn.disabled = true;
  btn.textContent = text.creating;

  try {
    console.log("🔐 Creating passkey...");

    // Use SDK to register passkey
    const result = await registerNewPasskey({
      userName: userName.toLowerCase().replace(/\s+/g, ""),
      userDisplayName: userName,
    });

    console.log("✅ Passkey created successfully!");
    console.log(`Credential ID: ${result.credentialId}`);

    // Store credentials
    passkeyCredentials = {
      credentialId: result.credentialId,
      credentialPublicKey: Array.from(result.credentialPublicKey),
      userName: userName.toLowerCase().replace(/\s+/g, ""),
      userDisplayName: userName,
    };

    savePasskeyData();

    // Update UI
    document.getElementById("passkey-input").classList.add("hidden");
    document.getElementById("passkey-success").classList.remove("hidden");
    document.getElementById("credentialIdDisplay").textContent = result.credentialId;

    // Update PayPal UI step 1
    const passkeyStep = document.getElementById("passkey-step");
    const passkeyIcon = document.getElementById("passkey-icon");
    if (passkeyStep) passkeyStep.classList.add("completed");
    if (passkeyIcon) {
      passkeyIcon.classList.add("completed");
      passkeyIcon.textContent = "✓";
    }

    // Enable deploy button
    document.getElementById("deployAccountBtn").disabled = false;
  } catch (error) {
    console.error("Passkey creation failed:", error);
    document.getElementById("passkey-error").textContent = `${text.failedToCreateKey} ${error.message}`;
    document.getElementById("passkey-error").classList.remove("hidden");
  } finally {
    btn.disabled = false;
    btn.textContent = text.createKey;
  }
}

// Step 2: Deploy Account
async function handleDeployAccount() {
  const btn = document.getElementById("deployAccountBtn");
  const text = translatedText();
  btn.disabled = true;
  btn.textContent = text.deploying;

  try {
    console.log("🚀 Deploying smart account...");

    // Create deployer client
    const deployerAccount = privateKeyToAccount(DEPLOYER_PRIVATE_KEY);
    const deployerClient = createWalletClient({
      account: deployerAccount,
      chain: zksyncOsTestnet,
      transport: http(ZKSYNC_OS_RPC_URL),
    });

    console.log(`Deployer address: ${deployerAccount.address}`);
    console.log(`Current origin: ${window.location.origin}`);
    console.log(`Credential ID: ${passkeyCredentials.credentialId}`);

    // Generate account ID from credential
    // Credential ID is base64url string, convert to bytes
    const { base64UrlToUint8Array } = await import("zksync-sso/utils");
    const credentialIdBytes = base64UrlToUint8Array(passkeyCredentials.credentialId);
    const credentialIdHex = toHex(credentialIdBytes);
    const accountId = keccak256(credentialIdHex);

    console.log(`Account ID: ${accountId}`);

    // Extract public key coordinates from credentialPublicKey using SDK's COSE parser
    const { getPublicKeyBytesFromPasskeySignature } = await import("zksync-sso/utils");
    const [xBytes, yBytes] = getPublicKeyBytesFromPasskeySignature(new Uint8Array(passkeyCredentials.credentialPublicKey));
    const x = "0x" + Array.from(xBytes).map((b) => b.toString(16).padStart(2, "0")).join("");
    const y = "0x" + Array.from(yBytes).map((b) => b.toString(16).padStart(2, "0")).join("");

    // Encode init data for WebAuthn validator
    // Format: (bytes credentialId, bytes32[2] publicKey, string domain)
    const webauthnInitData = encodeAbiParameters(
      parseAbiParameters("bytes, bytes32[2], string"),
      [credentialIdHex, [x, y], window.location.origin],
    );

    // Deploy WITHOUT initialization data to avoid the AlreadyInitialized error
    // We'll initialize manually after deployment
    const initData = "0x";

    console.log("Calling factory.deployAccount (without init)...");

    // Deploy the account using simple factory call
    const FACTORY_ABI = [
      {
        type: "function",
        name: "deployAccount",
        inputs: [
          { name: "accountId", type: "bytes32" },
          { name: "initData", type: "bytes" },
        ],
        outputs: [{ name: "account", type: "address" }],
        stateMutability: "nonpayable",
      },
      {
        type: "event",
        name: "AccountCreated",
        inputs: [
          { name: "account", type: "address", indexed: true },
          { name: "deployer", type: "address", indexed: true },
        ],
      },
    ];

    const hash = await writeContract(deployerClient, {
      address: CONTRACTS.accountFactory,
      abi: FACTORY_ABI,
      functionName: "deployAccount",
      args: [accountId, initData],
    });

    console.log(`Transaction hash: ${hash}`);
    console.log("Waiting for confirmation...");

    const receipt = await waitForTransactionReceipt(publicClient, { hash });

    if (receipt.status !== "success") {
      throw new Error("Account deployment transaction reverted");
    }

    // Parse logs to find AccountCreated event
    const logs = parseEventLogs({
      abi: FACTORY_ABI,
      logs: receipt.logs,
    });

    const accountCreatedLog = logs.find((log) => log.eventName === "AccountCreated");
    if (accountCreatedLog) {
      accountAddress = accountCreatedLog.args.account;
    } else {
      throw new Error("No AccountCreated event found");
    }

    savePasskeyData();

    console.log("✅ Account deployed successfully!");
    console.log(`Account address: ${accountAddress}`);
    console.log(`Block: ${receipt.blockNumber}`);
    console.log(`Gas used: ${receipt.gasUsed}`);

    // Check if account is initialized
    console.log("Checking if account is initialized...");
    try {
      await publicClient.readContract({
        address: accountAddress,
        abi: [{
          type: "function",
          name: "listModuleValidators",
          inputs: [],
          outputs: [{ name: "validatorList", type: "address[]" }],
          stateMutability: "view",
        }],
        functionName: "listModuleValidators",
      });
      console.log("✅ Account is already initialized");
    } catch (error) {
      if (error.message.includes("NotInitialized") || error.message.includes("0x48c9ceda")) {
        console.log("⚠️  Account not initialized! Initializing now...");

        // Call initializeAccount directly
        const initHash = await writeContract(deployerClient, {
          address: accountAddress,
          abi: [{
            type: "function",
            name: "initializeAccount",
            inputs: [
              { name: "modules", type: "address[]" },
              { name: "data", type: "bytes[]" },
            ],
            stateMutability: "nonpayable",
          }],
          functionName: "initializeAccount",
          args: [[CONTRACTS.passkey], [webauthnInitData]],
        });

        console.log(`Initialization tx: ${initHash}`);
        const initReceipt = await waitForTransactionReceipt(publicClient, { hash: initHash });

        if (initReceipt.status !== "success") {
          throw new Error("Account initialization failed");
        }

        console.log("✅ Account initialized successfully!");
      } else {
        console.warn("Could not check initialization status:", error.message);
      }
    }

    // Update UI
    document.getElementById("deploy-button-container").classList.add("hidden");
    document.getElementById("deploy-success").classList.remove("hidden");
    document.getElementById("accountAddressDisplay").textContent = accountAddress;

    // Update PayPal UI step 2
    const deployStep = document.getElementById("deploy-step");
    const deployIcon = document.getElementById("deploy-icon");
    if (deployStep) deployStep.classList.add("completed");
    if (deployIcon) {
      deployIcon.classList.add("completed");
      deployIcon.textContent = "✓";
    }

    // Show wallet view instead of setup section
    const setupSection = document.getElementById("setup-section");
    const walletView = document.getElementById("wallet-view");
    if (setupSection) setupSection.classList.add("hidden");
    if (walletView) walletView.classList.remove("hidden");

    // Update wallet address display
    const walletAddressDisplay = document.getElementById("walletAddressDisplay");
    if (walletAddressDisplay) walletAddressDisplay.textContent = accountAddress;

    // Enable transfer
    document.getElementById("transferBtn").disabled = false;
    document.getElementById("sendMaxBtn").disabled = false;

    // Enable Aave buttons
    const aaveDepositBtn = document.getElementById("aaveDepositBtn");
    const aaveWithdrawBtn = document.getElementById("aaveWithdrawBtn");
    if (aaveDepositBtn) aaveDepositBtn.disabled = false;
    if (aaveWithdrawBtn) aaveWithdrawBtn.disabled = false;

    startBalanceAutoRefresh();
    initializeAaveSection();
    startActivityPoll();
  } catch (error) {
    console.error("Deployment failed:", error);
    document.getElementById("deploy-error").textContent = `${text.deployFailed} ${error.message}`;
    document.getElementById("deploy-error").classList.remove("hidden");
  } finally {
    btn.disabled = false;
    btn.textContent = text.deployAccount;
  }
}

// Auto-refresh balance in the background
async function autoRefreshBalance() {
  if (!accountAddress) return;

  try {
    const balance = await publicClient.getBalance({
      address: accountAddress,
      blockTag: "latest", // Force fetching the latest block
    });

    const balanceInEth = formatEther(balance);
    document.getElementById("balanceDisplay").textContent = balanceInEth;
  } catch (error) {
    console.error("Auto-refresh balance failed:", error);
  }
}

// Start auto-refresh
function startBalanceAutoRefresh() {
  // Clear any existing interval
  if (balanceRefreshInterval) {
    clearInterval(balanceRefreshInterval);
  }

  // Refresh immediately
  autoRefreshBalance();

  // Then refresh every 5 seconds
  balanceRefreshInterval = setInterval(autoRefreshBalance, BALANCE_REFRESH_INTERVAL);
  console.log("✅ Balance auto-refresh started");
}

// // Stop auto-refresh
// function stopBalanceAutoRefresh() {
//   if (balanceRefreshInterval) {
//     clearInterval(balanceRefreshInterval);
//     balanceRefreshInterval = null;
//     console.log("⏹️  Balance auto-refresh stopped");
//   }
// }

// Step 3: Refresh Balance (manual)
async function handleRefreshBalance() {
  if (!accountAddress) return;

  const text = translatedText();
  const btn = document.getElementById("refreshBalanceBtn");
  btn.disabled = true;
  btn.textContent = text.refreshing;

  try {
    await autoRefreshBalance();
  } catch (error) {
    console.error("Balance fetch failed:", error);
  } finally {
    btn.disabled = false;
    btn.textContent = text.refreshBalance;
  }
}

// Faucet function to fund the account
async function handleFaucet() {
  if (!accountAddress) return;

  const text = translatedText();
  const btn = document.getElementById("faucetBtn");
  btn.disabled = true;
  btn.textContent = text.funding;

  try {
    console.log("🚰 Starting faucet for account:", accountAddress);

    // Create deployer wallet client for L2
    const deployerAccount = privateKeyToAccount(DEPLOYER_PRIVATE_KEY);
    const deployerWallet = createWalletClient({
      account: deployerAccount,
      chain: zksyncOsTestnet,
      transport: http(ZKSYNC_OS_RPC_URL),
    });

    const FAUCET_AMOUNT = parseEther("0.1");

    // Transaction 1: Deposit to EntryPoint
    console.log("📥 Depositing to EntryPoint...");
    const depositHash = await deployerWallet.writeContract({
      address: CONTRACTS.entryPoint,
      abi: [{
        type: "function",
        name: "depositTo",
        inputs: [{ name: "account", type: "address" }],
        outputs: [],
        stateMutability: "payable",
      }],
      functionName: "depositTo",
      args: [accountAddress],
      value: FAUCET_AMOUNT,
    });

    console.log(`✅ EntryPoint deposit tx: ${depositHash}`);
    await waitForTransactionReceipt(publicClient, { hash: depositHash });

    // Transaction 2: Direct ETH transfer to account
    console.log("💸 Sending ETH to account...");
    const transferHash = await deployerWallet.sendTransaction({
      to: accountAddress,
      value: FAUCET_AMOUNT,
    });

    console.log(`✅ Direct transfer tx: ${transferHash}`);
    await waitForTransactionReceipt(publicClient, { hash: transferHash });

    // Transaction 3: Fund shadow account on Sepolia
    console.log("🌉 Funding shadow account on Sepolia...");
    const shadowAccount = await getShadowAccount(publicClient, accountAddress);
    console.log(`Shadow account: ${shadowAccount}`);

    const sepoliaDeployerWallet = createWalletClient({
      account: deployerAccount,
      chain: sepolia,
      transport: http("https://eth-sepolia.g.alchemy.com/v2/Oa5oz2Y9QWGrxv8_0tqabXz_RFc0tqLU"),
    });

    const SHADOW_AMOUNT = parseEther("0.03");
    const shadowTransferHash = await sepoliaDeployerWallet.sendTransaction({
      to: shadowAccount,
      value: SHADOW_AMOUNT,
    });

    console.log(`✅ Shadow account funding tx: ${shadowTransferHash}`);
    await waitForTransactionReceipt(sepoliaClient, { hash: shadowTransferHash });

    console.log("🎉 Faucet complete! Funded with 0.23 ETH total (0.2 on L2 + 0.03 on Sepolia shadow account)");

    // Refresh balance
    await autoRefreshBalance();

    alert(text.faucetSuccess);
  } catch (error) {
    console.error("Faucet failed:", error);
    alert(`${text.faucetFailed} ${error.message}`);
  } finally {
    btn.disabled = false;
    btn.textContent = text.faucet;
  }
}

// Ensure account is properly initialized with passkey registered
async function ensureAccountInitialized() {
  console.log("🔍 Checking if passkey is registered...");

  // Check if the passkey is registered in the WebAuthn validator
  // Credential ID is base64url string, convert to bytes
  const { base64UrlToUint8Array } = await import("zksync-sso/utils");
  const credentialIdBytes = base64UrlToUint8Array(passkeyCredentials.credentialId);
  const credentialIdHex = toHex(credentialIdBytes);

  try {
    const registeredKey = await publicClient.readContract({
      address: CONTRACTS.passkey,
      abi: [{
        type: "function",
        name: "getAccountKey",
        inputs: [
          { name: "domain", type: "string" },
          { name: "credentialId", type: "bytes" },
          { name: "account", type: "address" },
        ],
        outputs: [{ name: "", type: "bytes32[2]" }],
        stateMutability: "view",
      }],
      functionName: "getAccountKey",
      args: [window.location.origin, credentialIdHex, accountAddress],
    });

    if (registeredKey[0] === "0x0000000000000000000000000000000000000000000000000000000000000000"
      && registeredKey[1] === "0x0000000000000000000000000000000000000000000000000000000000000000") {
      console.error("❌ Passkey is NOT registered!");
      console.error("Account is in an invalid state. Please reset and redeploy.");
      throw new Error("Passkey not registered. The account is in an invalid state. Please click \"Reset Passkey\" and create a new account.");
    }

    console.log("✅ Passkey is registered");
    return true;
  } catch (error) {
    if (error.message.includes("Passkey not registered")) {
      throw error;
    }
    console.error("Failed to check passkey registration:", error);
    throw new Error("Failed to verify passkey registration: " + error.message);
  }
}

// Send maximum balance (entire balance can be sent since gas is paid via ERC-4337)
async function handleSendMax() {
  const text = translatedText();
  try {
    if (!accountAddress) {
      alert(text.deployAccountAlert);
      return;
    }

    const balance = await publicClient.getBalance({ address: accountAddress });
    const maxEth = formatEther(balance);

    document.getElementById("transferAmount").value = maxEth;
    document.getElementById("max-amount-info").textContent
      = `${text.sendingBalance} ${maxEth} ETH (${text.gasPaid})`;
  } catch (error) {
    console.error("Failed to get balance:", error);
    alert(text.balanceFailed);
  }
}

// Step 4: Transfer ETH using ERC-4337 bundler
async function handleTransfer() {
  const recipient = document.getElementById("recipientAddress").value.trim();
  const amount = document.getElementById("transferAmount").value.trim();
  const text = translatedText();

  if (!recipient || !amount) {
    alert(text.inputError);
    return;
  }

  const btn = document.getElementById("transferBtn");
  btn.disabled = true;
  btn.textContent = text.transferring;

  // Hide previous results
  document.getElementById("transfer-success").classList.add("hidden");
  document.getElementById("transfer-error").classList.add("hidden");

  try {
    // First, ensure account is initialized
    await ensureAccountInitialized();

    console.log("💸 Preparing transfer...");
    console.log(`To: ${recipient}`);
    console.log(`Amount: ${amount} ETH`);

    // Use the SDK's requestPasskeyAuthentication for signing
    const { requestPasskeyAuthentication } = await import("zksync-sso/client/passkey");

    // Create UserOperation for ETH transfer
    // Use ERC-7579 execute(bytes32 mode, bytes executionData) format
    const modeCode = pad("0x01", { dir: "right", size: 32 }); // simple batch execute

    // Encode execution data as Call[] array
    const executionData = encodeAbiParameters(
      [{
        components: [
          { name: "to", type: "address" },
          { name: "value", type: "uint256" },
          { name: "data", type: "bytes" },
        ],
        name: "Call",
        type: "tuple[]",
      }],
      [[{
        to: recipient,
        value: parseEther(amount),
        data: "0x",
      }]],
    );

    // Encode execute(bytes32,bytes) call
    const callData = concat([
      "0xe9ae5c53", // execute(bytes32,bytes) selector
      encodeAbiParameters(
        [{ type: "bytes32" }, { type: "bytes" }],
        [modeCode, executionData],
      ),
    ]);

    // Get nonce from EntryPoint
    const ENTRYPOINT_ABI = [{
      type: "function",
      name: "getNonce",
      inputs: [
        { name: "sender", type: "address" },
        { name: "key", type: "uint192" },
      ],
      outputs: [{ name: "nonce", type: "uint256" }],
      stateMutability: "view",
    }];

    const nonce = await publicClient.readContract({
      address: CONTRACTS.entryPoint,
      abi: ENTRYPOINT_ABI,
      functionName: "getNonce",
      args: [accountAddress, 0n],
    });

    console.log(`Nonce: ${nonce}`);

    // ERC-4337 v0.8 uses packed gas limits
    const callGasLimit = 500000n; // Increased from 200000
    const verificationGasLimit = 2000000n; // Increased from 500000
    const maxFeePerGas = 10000000000n; // 10 gwei (increased from 2)
    const maxPriorityFeePerGas = 5000000000n; // 5 gwei (increased from 1)
    const preVerificationGas = 200000n; // Increased from 100000

    // Pack gas limits: (verificationGasLimit << 128) | callGasLimit
    const accountGasLimits = pad(toHex((verificationGasLimit << 128n) | callGasLimit), { size: 32 });

    // Pack gas fees: (maxPriorityFeePerGas << 128) | maxFeePerGas
    const gasFees = pad(toHex((maxPriorityFeePerGas << 128n) | maxFeePerGas), { size: 32 });

    // Create PackedUserOperation for v0.8
    const packedUserOp = {
      sender: accountAddress,
      nonce,
      initCode: "0x",
      callData,
      accountGasLimits,
      preVerificationGas,
      gasFees,
      paymasterAndData: "0x",
      signature: "0x",
    };

    // Calculate UserOperation hash manually using EIP-712 for v0.8
    const PACKED_USEROP_TYPEHASH = "0x29a0bca4af4be3421398da00295e58e6d7de38cb492214754cb6a47507dd6f8e";

    // EIP-712 domain separator - use toBytes for proper string encoding
    const domainTypeHash = "0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f";
    const nameHash = keccak256(toBytes("ERC4337"));
    const versionHash = keccak256(toBytes("1"));

    const domainSeparator = keccak256(encodeAbiParameters(
      parseAbiParameters("bytes32,bytes32,bytes32,uint256,address"),
      [domainTypeHash, nameHash, versionHash, BigInt(zksyncOsTestnet.id), CONTRACTS.entryPoint],
    ));

    // Hash the PackedUserOperation struct
    const structHash = keccak256(encodeAbiParameters(
      parseAbiParameters("bytes32,address,uint256,bytes32,bytes32,bytes32,uint256,bytes32,bytes32"),
      [
        PACKED_USEROP_TYPEHASH,
        packedUserOp.sender,
        packedUserOp.nonce,
        keccak256(packedUserOp.initCode),
        keccak256(packedUserOp.callData),
        packedUserOp.accountGasLimits,
        packedUserOp.preVerificationGas,
        packedUserOp.gasFees,
        keccak256(packedUserOp.paymasterAndData),
      ],
    ));

    // Final EIP-712 hash
    const userOpHash = keccak256(concat(["0x1901", domainSeparator, structHash]));

    console.log(`UserOp hash (EIP-712 v0.8): ${userOpHash}`);
    console.log("Domain separator:", domainSeparator);
    console.log("Struct hash:", structHash);
    console.log("Account gas limits:", packedUserOp.accountGasLimits);
    console.log("Gas fees:", packedUserOp.gasFees);
    console.log("CallData:", callData);
    console.log("🔐 Requesting passkey authentication...");

    // Sign with passkey
    const passkeySignature = await requestPasskeyAuthentication({
      challenge: userOpHash,
      credentialPublicKey: new Uint8Array(passkeyCredentials.credentialPublicKey),
    });

    // Parse signature using SDK utilities
    const response = passkeySignature.passkeyAuthenticationResponse.response;

    // Decode base64url encoded data
    const authenticatorDataHex = toHex(base64UrlToUint8Array(response.authenticatorData));
    const credentialIdHex = toHex(base64UrlToUint8Array(passkeySignature.passkeyAuthenticationResponse.id));

    // Parse DER signature using SDK's unwrapEC2Signature
    const signatureData = unwrapEC2Signature(base64UrlToUint8Array(response.signature));

    // Ensure r and s are exactly 32 bytes (left-padded with zeros if needed)
    const r = pad(toHex(signatureData.r), { size: 32 });
    const s = pad(toHex(signatureData.s), { size: 32 });

    console.log("Parsed r:", r);
    console.log("Parsed s:", s);
    console.log("r length:", r.length, "bytes:", (r.length - 2) / 2);
    console.log("s length:", s.length, "bytes:", (s.length - 2) / 2);

    // Encode signature for ERC-4337 bundler (matching test format)
    const passkeySignatureEncoded = encodeAbiParameters(
      [
        { type: "bytes" }, // authenticatorData
        { type: "string" }, // clientDataJSON
        { type: "bytes32[2]" }, // r and s as array
        { type: "bytes" }, // credentialId
      ],
      [
        authenticatorDataHex,
        new TextDecoder().decode(base64UrlToUint8Array(response.clientDataJSON)),
        [r, s],
        credentialIdHex,
      ],
    );

    // Prepend validator address (ERC-4337 format)
    packedUserOp.signature = concat([CONTRACTS.passkey, passkeySignatureEncoded]);

    console.log("Authenticator data:", authenticatorDataHex);
    console.log("Client data JSON:", new TextDecoder().decode(base64UrlToUint8Array(response.clientDataJSON)));
    console.log("Credential ID:", credentialIdHex);
    console.log("Signature (r, s):", r, s);
    console.log("Full signature length:", packedUserOp.signature.length);
    console.log("📤 Submitting UserOperation to bundler...");

    // Submit v0.8 packed format to bundler
    const userOpForBundler = {
      sender: packedUserOp.sender,
      nonce: toHex(packedUserOp.nonce),
      factory: null, // No factory since account already deployed
      factoryData: null,
      callData: packedUserOp.callData,
      callGasLimit: toHex(callGasLimit),
      verificationGasLimit: toHex(verificationGasLimit),
      preVerificationGas: toHex(preVerificationGas),
      maxFeePerGas: toHex(maxFeePerGas),
      maxPriorityFeePerGas: toHex(maxPriorityFeePerGas),
      paymaster: null, // No paymaster
      paymasterVerificationGasLimit: null,
      paymasterPostOpGasLimit: null,
      paymasterData: null,
      signature: packedUserOp.signature,
    };

    // Submit to bundler (v0.8 RPC format)
    const bundlerResponse = await fetch(CONTRACTS.bundlerUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "eth_sendUserOperation",
        params: [userOpForBundler, CONTRACTS.entryPoint],
      }),
    });

    const bundlerResult = await bundlerResponse.json();

    if (bundlerResult.error) {
      throw new Error(`Bundler error: ${bundlerResult.error.message}`);
    }

    const userOpHashFromBundler = bundlerResult.result;
    console.log(`UserOperation submitted: ${userOpHashFromBundler}`);
    console.log("⏳ Waiting for confirmation...");

    // Poll for receipt
    let receipt = null;
    for (let i = 0; i < 30; i++) {
      await new Promise((resolve) => setTimeout(resolve, 2000));

      const receiptResponse = await fetch(CONTRACTS.bundlerUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: 1,
          method: "eth_getUserOperationReceipt",
          params: [userOpHashFromBundler],
        }),
      });

      const receiptResult = await receiptResponse.json();
      if (receiptResult.result) {
        receipt = receiptResult.result;
        break;
      }
    }

    if (!receipt) {
      throw new Error("Transaction timeout - could not get receipt");
    }

    if (receipt.success) {
      console.log("✅ Transfer successful!");
      console.log(`Transaction hash: ${receipt.receipt.transactionHash}`);
      console.log(`Block: ${receipt.receipt.blockNumber}`);

      // Update UI
      document.getElementById("transfer-success").classList.remove("hidden");
      document.getElementById("txHashDisplay").textContent = receipt.receipt.transactionHash;

      // Set explorer link
      const txLink = document.getElementById("transfer-tx-link");
      txLink.href = `${L2_EXPLORER_BASE}${receipt.receipt.transactionHash}`;

      // Refresh balance
      await handleRefreshBalance();

      // Clear form
      document.getElementById("recipientAddress").value = "";
      document.getElementById("transferAmount").value = "";
    } else {
      throw new Error("Transaction failed");
    }
  } catch (error) {
    console.error("Transfer failed:", error);
    document.getElementById("transfer-error").textContent = `${text.transferFailed} ${error.message}`;
    document.getElementById("transfer-error").classList.remove("hidden");
  } finally {
    btn.disabled = false;
    btn.textContent = text.sendETH;
  }
}

// ===== AAVE INTEGRATION =====

// Initialize Aave section when account is deployed
async function initializeAaveSection() {
  if (!accountAddress) return;

  try {
    console.log("🔵 Initializing Aave section...");

    // Get shadow account address
    shadowAccount = await getShadowAccount(publicClient, accountAddress);
    console.log(`Shadow account: ${shadowAccount}`);

    // Update UI
    document.getElementById("shadowAccountDisplay").textContent = shadowAccount;
    document.getElementById("aave-balance-section").classList.remove("hidden");

    // Enable Aave buttons
    document.getElementById("aaveDepositBtn").disabled = false;
    document.getElementById("aaveWithdrawBtn").disabled = false;
    document.getElementById("refreshAaveBalanceBtn").disabled = false;

    // Refresh Aave balance
    await refreshAaveBalance();

    // Start auto-refresh for Aave balance
    startAaveBalanceAutoRefresh();

    console.log("✅ Aave section initialized");
  } catch (error) {
    console.error("Failed to initialize Aave section:", error);
  }
}

// Silent auto-refresh for Aave balance (no UI feedback)
async function autoRefreshAaveBalance() {
  if (!shadowAccount) return;

  try {
    const balance = await getAaveBalance(sepoliaClient, shadowAccount);
    document.getElementById("aaveBalanceDisplay").textContent = formatEther(balance);
  } catch (error) {
    console.error("Auto-refresh Aave balance failed:", error);
  }
}

// Start Aave balance auto-refresh
function startAaveBalanceAutoRefresh() {
  if (aaveBalanceRefreshInterval) {
    clearInterval(aaveBalanceRefreshInterval);
  }

  // Initial refresh
  autoRefreshAaveBalance();

  // Set up interval
  aaveBalanceRefreshInterval = setInterval(autoRefreshAaveBalance, AAVE_BALANCE_REFRESH_INTERVAL);
  console.log("✅ Aave balance auto-refresh started");
}

// // Stop Aave balance auto-refresh
// function stopAaveBalanceAutoRefresh() {
//   if (aaveBalanceRefreshInterval) {
//     clearInterval(aaveBalanceRefreshInterval);
//     aaveBalanceRefreshInterval = null;
//     console.log("⏹️  Aave balance auto-refresh stopped");
//   }
// }

// Refresh Aave balance (aToken balance on L1)
async function refreshAaveBalance() {
  if (!shadowAccount) return;

  const text = translatedText();
  const btn = document.getElementById("refreshAaveBalanceBtn");
  if (btn) {
    btn.disabled = true;
    btn.textContent = text.refreshing;
  }

  try {
    console.log("🔵 Checking Aave balance on L1 Sepolia...");

    const balance = await getAaveBalance(sepoliaClient, shadowAccount);
    console.log(`Aave balance: ${formatEther(balance)} aETH`);

    // Update UI
    document.getElementById("aaveBalanceDisplay").textContent = formatEther(balance);
  } catch (error) {
    console.error("Failed to refresh Aave balance:", error);
    document.getElementById("aaveBalanceDisplay").textContent = text.balanceError;
  } finally {
    if (btn) {
      btn.disabled = false;
      btn.textContent = text.refreshBalance;
    }
  }
}

// Handle Aave deposit
async function handleAaveDeposit() {
  if (!accountAddress || !shadowAccount) return;

  const btn = document.getElementById("aaveDepositBtn");
  const text = translatedText();
  btn.disabled = true;
  btn.textContent = text.depositing;

  // Hide previous messages
  document.getElementById("aave-deposit-success").classList.add("hidden");
  document.getElementById("aave-deposit-error").classList.add("hidden");

  try {
    // First, ensure account is initialized
    await ensureAccountInitialized();

    const amount = document.getElementById("aaveDepositAmount").value;
    if (!amount || parseFloat(amount) <= 0) {
      throw new Error("Please enter a valid amount");
    }

    const amountWei = parseEther(amount);
    console.log(`🔵 Initiating Aave deposit of ${amount} ETH...`);

    // Use the SDK's requestPasskeyAuthentication for signing
    const { requestPasskeyAuthentication } = await import("zksync-sso/client/passkey");

    // Create Aave deposit bundle
    const { bundle, withdrawCall } = await createAaveDepositBundle(amountWei, shadowAccount);
    console.log("Bundle created:", bundle);

    // Encode the call to L2InteropCenter.sendBundleToL1
    const l2InteropCallData = encodeFunctionData({
      abi: bundle.abi,
      functionName: bundle.functionName,
      args: bundle.args,
    });

    // Wrap in ERC-7579 execute(bytes32 mode, bytes executionData) format
    const modeCode = pad("0x01", { dir: "right", size: 32 }); // simple batch execute

    // Encode execution data as Call[] array
    const executionData = encodeAbiParameters(
      [{
        components: [
          { name: "to", type: "address" },
          { name: "value", type: "uint256" },
          { name: "data", type: "bytes" },
        ],
        name: "Call",
        type: "tuple[]",
      }],
      [[
        {
          to: withdrawCall.to,
          value: withdrawCall.value,
          data: withdrawCall.data,
        },
        {
          to: AAVE_CONTRACTS.l2InteropCenter,
          value: 0n,
          data: l2InteropCallData,
        },
      ]],
    );

    // Encode execute(bytes32,bytes) call
    const callData = concat([
      "0xe9ae5c53", // execute(bytes32,bytes) selector
      encodeAbiParameters(
        [{ type: "bytes32" }, { type: "bytes" }],
        [modeCode, executionData],
      ),
    ]);

    // Get current nonce
    const nonce = await publicClient.readContract({
      address: CONTRACTS.entryPoint,
      abi: [
        {
          type: "function",
          name: "getNonce",
          inputs: [
            { name: "sender", type: "address" },
            { name: "key", type: "uint192" },
          ],
          outputs: [{ name: "nonce", type: "uint256" }],
          stateMutability: "view",
        },
      ],
      functionName: "getNonce",
      args: [accountAddress, 0n],
    });

    console.log(`Nonce: ${nonce}`);

    // Get gas price
    const feeData = await publicClient.estimateFeesPerGas();
    const maxFeePerGas = feeData.maxFeePerGas;
    const maxPriorityFeePerGas = feeData.maxPriorityFeePerGas || (maxFeePerGas * 5n) / 100n;

    // Gas limits for Aave operation (L2-to-L1 requires more gas)
    const callGasLimit = BigInt(3_000_000);
    const verificationGasLimit = BigInt(1_500_000);
    const preVerificationGas = BigInt(100_000);

    // Pack gas limits (v0.8 format)
    const accountGasLimits = concat([
      pad(toHex(verificationGasLimit), { size: 16 }),
      pad(toHex(callGasLimit), { size: 16 }),
    ]);

    const gasFees = concat([
      pad(toHex(maxPriorityFeePerGas), { size: 16 }),
      pad(toHex(maxFeePerGas), { size: 16 }),
    ]);

    // Create PackedUserOperation for v0.8
    const packedUserOp = {
      sender: accountAddress,
      nonce,
      initCode: "0x",
      callData,
      accountGasLimits,
      preVerificationGas,
      gasFees,
      paymasterAndData: "0x",
      signature: "0x",
    };

    // Calculate UserOperation hash manually using EIP-712 for v0.8
    const PACKED_USEROP_TYPEHASH = "0x29a0bca4af4be3421398da00295e58e6d7de38cb492214754cb6a47507dd6f8e";

    // EIP-712 domain separator
    const domainTypeHash = "0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f";
    const nameHash = keccak256(toBytes("ERC4337"));
    const versionHash = keccak256(toBytes("1"));

    const domainSeparator = keccak256(encodeAbiParameters(
      parseAbiParameters("bytes32,bytes32,bytes32,uint256,address"),
      [domainTypeHash, nameHash, versionHash, BigInt(zksyncOsTestnet.id), CONTRACTS.entryPoint],
    ));

    // Hash the PackedUserOperation struct
    const structHash = keccak256(encodeAbiParameters(
      parseAbiParameters("bytes32,address,uint256,bytes32,bytes32,bytes32,uint256,bytes32,bytes32"),
      [
        PACKED_USEROP_TYPEHASH,
        packedUserOp.sender,
        packedUserOp.nonce,
        keccak256(packedUserOp.initCode),
        keccak256(packedUserOp.callData),
        packedUserOp.accountGasLimits,
        packedUserOp.preVerificationGas,
        packedUserOp.gasFees,
        keccak256(packedUserOp.paymasterAndData),
      ],
    ));

    // EIP-712 typed data hash
    const userOpHash = keccak256(concat(["0x1901", domainSeparator, structHash]));

    console.log("🔐 Requesting passkey authentication for Aave deposit...");

    // Sign with passkey
    const passkeySignature = await requestPasskeyAuthentication({
      challenge: userOpHash,
      credentialPublicKey: new Uint8Array(passkeyCredentials.credentialPublicKey),
      authenticatorSelection: {},
      timeout: 60000,
    });

    // Parse signature
    const response = passkeySignature.passkeyAuthenticationResponse.response;
    const authenticatorDataHex = toHex(base64UrlToUint8Array(response.authenticatorData));
    const credentialIdHex = toHex(base64UrlToUint8Array(passkeySignature.passkeyAuthenticationResponse.id));
    const signatureData = unwrapEC2Signature(base64UrlToUint8Array(response.signature));

    const r = pad(toHex(signatureData.r), { size: 32 });
    const s = pad(toHex(signatureData.s), { size: 32 });

    // Encode signature
    const passkeySignatureEncoded = encodeAbiParameters(
      [
        { type: "bytes" },
        { type: "string" },
        { type: "bytes32[2]" },
        { type: "bytes" },
      ],
      [
        authenticatorDataHex,
        new TextDecoder().decode(base64UrlToUint8Array(response.clientDataJSON)),
        [r, s],
        credentialIdHex,
      ],
    );

    packedUserOp.signature = concat([CONTRACTS.passkey, passkeySignatureEncoded]);

    console.log("📤 Submitting Aave deposit UserOperation to bundler...");

    // Submit to bundler
    const userOpForBundler = {
      sender: packedUserOp.sender,
      nonce: toHex(packedUserOp.nonce),
      factory: null,
      factoryData: null,
      callData: packedUserOp.callData,
      callGasLimit: toHex(callGasLimit),
      verificationGasLimit: toHex(verificationGasLimit),
      preVerificationGas: toHex(preVerificationGas),
      maxFeePerGas: toHex(maxFeePerGas),
      maxPriorityFeePerGas: toHex(maxPriorityFeePerGas),
      paymaster: null,
      paymasterVerificationGasLimit: null,
      paymasterPostOpGasLimit: null,
      paymasterData: null,
      signature: packedUserOp.signature,
    };

    const bundlerResponse = await fetch(CONTRACTS.bundlerUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "eth_sendUserOperation",
        params: [userOpForBundler, CONTRACTS.entryPoint],
      }),
    });

    const bundlerResult = await bundlerResponse.json();

    if (bundlerResult.error) {
      throw new Error(`Bundler error: ${bundlerResult.error.message}`);
    }

    const userOpHashFromBundler = bundlerResult.result;
    console.log(`UserOperation submitted: ${userOpHashFromBundler}`);
    console.log("⏳ Waiting for L2 confirmation (this may take a few seconds)...");

    // Poll for receipt
    let receipt = null;
    for (let i = 0; i < 30; i++) {
      await new Promise((resolve) => setTimeout(resolve, 2000));

      const receiptResponse = await fetch(CONTRACTS.bundlerUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: 1,
          method: "eth_getUserOperationReceipt",
          params: [userOpHashFromBundler],
        }),
      });

      const receiptResult = await receiptResponse.json();
      if (receiptResult.result) {
        receipt = receiptResult.result;
        break;
      }
    }

    if (!receipt) {
      throw new Error("Transaction timeout - could not get receipt");
    }

    if (receipt.success) {
      console.log("✅ L2 transaction successful!");
      console.log(`Transaction hash: ${receipt.receipt.transactionHash}`);
      console.log("⏳ L2-to-L1 message sent. Finalization takes ~15 minutes...");

      // Save transaction metadata to localStorage
      saveTxMetadata(receipt.receipt.transactionHash, "Deposit", amount);

      // Update UI
      document.getElementById("aave-deposit-success").classList.remove("hidden");
      {
        const txEl = document.getElementById("aaveDepositTxHash");
        txEl.textContent = "";
        const link = document.createElement("a");
        link.href = `${L2_EXPLORER_BASE}${receipt.receipt.transactionHash}`;
        link.target = "_blank";
        link.rel = "noreferrer";
        link.textContent = receipt.receipt.transactionHash;
        txEl.appendChild(link);
      }

      // Clear form
      document.getElementById("aaveDepositAmount").value = "";

      // Note: Balance will update after L1 finalization (~15 min)
      console.log("💡 Note: Aave balance will update after L2-to-L1 finalization (~15 minutes)");
    } else {
      throw new Error("Transaction failed");
    }
  } catch (error) {
    console.error("Aave deposit failed:", error);
    document.getElementById("aave-deposit-error").textContent = `${text.depositFailed} ${error.message}`;
    document.getElementById("aave-deposit-error").classList.remove("hidden");
  } finally {
    btn.disabled = false;
    btn.textContent = text.depositToAave;
  }
}

// Handle Aave withdraw
async function handleAaveWithdraw() {
  if (!accountAddress || !shadowAccount) return;

  const btn = document.getElementById("aaveWithdrawBtn");
  const text = translatedText();
  btn.disabled = true;
  btn.textContent = text.withdrawing;

  // Hide previous messages
  document.getElementById("aave-withdraw-success").classList.add("hidden");
  document.getElementById("aave-withdraw-error").classList.add("hidden");

  try {
    // First, ensure account is initialized
    await ensureAccountInitialized();

    const amount = document.getElementById("aaveWithdrawAmount").value;
    if (!amount || parseFloat(amount) <= 0) {
      throw new Error("Please enter a valid amount");
    }

    const amountWei = parseEther(amount);
    console.log(`🔵 Initiating Aave withdrawal of ${amount} aETH...`);

    // Use the SDK's requestPasskeyAuthentication for signing
    const { requestPasskeyAuthentication } = await import("zksync-sso/client/passkey");

    // Check balance first
    const balance = await getAaveBalance(sepoliaClient, shadowAccount);
    if (balance < amountWei) {
      throw new Error(`Insufficient Aave balance. You have ${formatEther(balance)} aETH`);
    }

    // Create Aave withdraw bundle
    const bundle = await createAaveWithdrawBundle(amountWei, shadowAccount, accountAddress, sepoliaClient);
    console.log("Withdraw bundle created:", bundle);

    // Encode the call to L2InteropCenter.sendBundleToL1
    const l2InteropCallData = encodeFunctionData({
      abi: bundle.abi,
      functionName: bundle.functionName,
      args: bundle.args,
    });

    // Wrap in ERC-7579 execute(bytes32 mode, bytes executionData) format
    const modeCode = pad("0x01", { dir: "right", size: 32 }); // simple batch execute

    // Encode execution data as Call[] array
    const executionData = encodeAbiParameters(
      [{
        components: [
          { name: "to", type: "address" },
          { name: "value", type: "uint256" },
          { name: "data", type: "bytes" },
        ],
        name: "Call",
        type: "tuple[]",
      }],
      [[{
        to: AAVE_CONTRACTS.l2InteropCenter,
        value: 0n,
        data: l2InteropCallData,
      }]],
    );

    // Encode execute(bytes32,bytes) call
    const callData = concat([
      "0xe9ae5c53", // execute(bytes32,bytes) selector
      encodeAbiParameters(
        [{ type: "bytes32" }, { type: "bytes" }],
        [modeCode, executionData],
      ),
    ]);

    // Get current nonce
    const nonce = await publicClient.readContract({
      address: CONTRACTS.entryPoint,
      abi: [
        {
          type: "function",
          name: "getNonce",
          inputs: [
            { name: "sender", type: "address" },
            { name: "key", type: "uint192" },
          ],
          outputs: [{ name: "nonce", type: "uint256" }],
          stateMutability: "view",
        },
      ],
      functionName: "getNonce",
      args: [accountAddress, 0n],
    });

    console.log(`Nonce: ${nonce}`);

    // Get gas price
    const feeData = await publicClient.estimateFeesPerGas();
    const maxFeePerGas = feeData.maxFeePerGas;
    const maxPriorityFeePerGas = feeData.maxPriorityFeePerGas || (maxFeePerGas * 5n) / 100n;

    // Gas limits for Aave operation
    const callGasLimit = BigInt(3_000_000);
    const verificationGasLimit = BigInt(1_500_000);
    const preVerificationGas = BigInt(100_000);

    // Pack gas limits (v0.8 format)
    const accountGasLimits = concat([
      pad(toHex(verificationGasLimit), { size: 16 }),
      pad(toHex(callGasLimit), { size: 16 }),
    ]);

    const gasFees = concat([
      pad(toHex(maxPriorityFeePerGas), { size: 16 }),
      pad(toHex(maxFeePerGas), { size: 16 }),
    ]);

    // Create PackedUserOperation for v0.8
    const packedUserOp = {
      sender: accountAddress,
      nonce,
      initCode: "0x",
      callData,
      accountGasLimits,
      preVerificationGas,
      gasFees,
      paymasterAndData: "0x",
      signature: "0x",
    };

    // Calculate UserOperation hash manually using EIP-712 for v0.8
    const PACKED_USEROP_TYPEHASH = "0x29a0bca4af4be3421398da00295e58e6d7de38cb492214754cb6a47507dd6f8e";

    // EIP-712 domain separator
    const domainTypeHash = "0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f";
    const nameHash = keccak256(toBytes("ERC4337"));
    const versionHash = keccak256(toBytes("1"));

    const domainSeparator = keccak256(encodeAbiParameters(
      parseAbiParameters("bytes32,bytes32,bytes32,uint256,address"),
      [domainTypeHash, nameHash, versionHash, BigInt(zksyncOsTestnet.id), CONTRACTS.entryPoint],
    ));

    // Hash the PackedUserOperation struct
    const structHash = keccak256(encodeAbiParameters(
      parseAbiParameters("bytes32,address,uint256,bytes32,bytes32,bytes32,uint256,bytes32,bytes32"),
      [
        PACKED_USEROP_TYPEHASH,
        packedUserOp.sender,
        packedUserOp.nonce,
        keccak256(packedUserOp.initCode),
        keccak256(packedUserOp.callData),
        packedUserOp.accountGasLimits,
        packedUserOp.preVerificationGas,
        packedUserOp.gasFees,
        keccak256(packedUserOp.paymasterAndData),
      ],
    ));

    // EIP-712 typed data hash
    const userOpHash = keccak256(concat(["0x1901", domainSeparator, structHash]));

    console.log("🔐 Requesting passkey authentication for Aave withdrawal...");

    // Sign with passkey
    const passkeySignature = await requestPasskeyAuthentication({
      challenge: userOpHash,
      credentialPublicKey: new Uint8Array(passkeyCredentials.credentialPublicKey),
      authenticatorSelection: {},
      timeout: 60000,
    });

    // Parse signature
    const response = passkeySignature.passkeyAuthenticationResponse.response;
    const authenticatorDataHex = toHex(base64UrlToUint8Array(response.authenticatorData));
    const credentialIdHex = toHex(base64UrlToUint8Array(passkeySignature.passkeyAuthenticationResponse.id));
    const signatureData = unwrapEC2Signature(base64UrlToUint8Array(response.signature));

    const r = pad(toHex(signatureData.r), { size: 32 });
    const s = pad(toHex(signatureData.s), { size: 32 });

    // Encode signature
    const passkeySignatureEncoded = encodeAbiParameters(
      [
        { type: "bytes" },
        { type: "string" },
        { type: "bytes32[2]" },
        { type: "bytes" },
      ],
      [
        authenticatorDataHex,
        new TextDecoder().decode(base64UrlToUint8Array(response.clientDataJSON)),
        [r, s],
        credentialIdHex,
      ],
    );

    packedUserOp.signature = concat([CONTRACTS.passkey, passkeySignatureEncoded]);

    console.log("📤 Submitting Aave withdrawal UserOperation to bundler...");

    // Submit to bundler
    const userOpForBundler = {
      sender: packedUserOp.sender,
      nonce: toHex(packedUserOp.nonce),
      factory: null,
      factoryData: null,
      callData: packedUserOp.callData,
      callGasLimit: toHex(callGasLimit),
      verificationGasLimit: toHex(verificationGasLimit),
      preVerificationGas: toHex(preVerificationGas),
      maxFeePerGas: toHex(maxFeePerGas),
      maxPriorityFeePerGas: toHex(maxPriorityFeePerGas),
      paymaster: null,
      paymasterVerificationGasLimit: null,
      paymasterPostOpGasLimit: null,
      paymasterData: null,
      signature: packedUserOp.signature,
    };

    const bundlerResponse = await fetch(CONTRACTS.bundlerUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "eth_sendUserOperation",
        params: [userOpForBundler, CONTRACTS.entryPoint],
      }),
    });

    const bundlerResult = await bundlerResponse.json();

    if (bundlerResult.error) {
      throw new Error(`Bundler error: ${bundlerResult.error.message}`);
    }

    const userOpHashFromBundler = bundlerResult.result;
    console.log(`UserOperation submitted: ${userOpHashFromBundler}`);
    console.log("⏳ Waiting for L2 confirmation (this may take a few seconds)...");

    // Poll for receipt
    let receipt = null;
    for (let i = 0; i < 30; i++) {
      await new Promise((resolve) => setTimeout(resolve, 2000));

      const receiptResponse = await fetch(CONTRACTS.bundlerUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: 1,
          method: "eth_getUserOperationReceipt",
          params: [userOpHashFromBundler],
        }),
      });

      const receiptResult = await receiptResponse.json();
      if (receiptResult.result) {
        receipt = receiptResult.result;
        break;
      }
    }

    if (!receipt) {
      throw new Error("Transaction timeout - could not get receipt");
    }

    if (receipt.success) {
      console.log("✅ L2 transaction successful!");
      console.log(`Transaction hash: ${receipt.receipt.transactionHash}`);
      console.log("⏳ L2-to-L1 message sent. Finalization takes ~15 minutes...");

      // Save transaction metadata to localStorage
      saveTxMetadata(receipt.receipt.transactionHash, "Withdrawal", amount);

      // Update UI
      document.getElementById("aave-withdraw-success").classList.remove("hidden");
      {
        const txEl = document.getElementById("aaveWithdrawTxHash");
        txEl.textContent = "";
        const link = document.createElement("a");
        link.href = `${L2_EXPLORER_BASE}${receipt.receipt.transactionHash}`;
        link.target = "_blank";
        link.rel = "noreferrer";
        link.textContent = receipt.receipt.transactionHash;
        txEl.appendChild(link);
      }

      // Clear form
      document.getElementById("aaveWithdrawAmount").value = "";

      // Note: Balance will update after L1 finalization (~15 min)
      console.log("💡 Note: Aave balance will update after L2-to-L1 finalization (~15 minutes)");
    } else {
      throw new Error("Transaction failed");
    }
  } catch (error) {
    console.error("Aave withdrawal failed:", error);
    document.getElementById("aave-withdraw-error").textContent = `${text.withdrawFailed} ${error.message}`;
    document.getElementById("aave-withdraw-error").classList.remove("hidden");
  } finally {
    btn.disabled = false;
    btn.textContent = text.withdrawFromAave;
  }
}

// Interop functionality
const CHAIN_A_RPC = "http://localhost:3050";
const CHAIN_B_RPC = "http://localhost:3051";

async function handleInteropSend() {
  const btn = document.getElementById("interopSendBtn");
  const message = document.getElementById("interopMessage").value;
  const directionSelect = document.getElementById("interopMessageDirection");
  const direction = directionSelect.value;

  // Hide previous results
  document.getElementById("interop-success").classList.add("hidden");
  document.getElementById("interop-error").classList.add("hidden");
  document.getElementById("interop-progress").classList.remove("hidden");

  const text = translatedText();

  btn.disabled = true;
  btn.textContent = text.sending;

  const progressSteps = document.getElementById("interopProgressSteps");
  progressSteps.innerHTML = "";

  const addProgressStep = (step) => {
    const stepEl = document.createElement("div");
    stepEl.textContent = step;
    stepEl.style.padding = "4px 0";
    stepEl.style.fontSize = "13px";
    progressSteps.appendChild(stepEl);
  };

  try {
    console.log("🌉 Starting interop message flow...");

    // Determine source and destination based on direction
    const isAtoB = direction === "a-to-b";
    const sourceRpc = isAtoB ? CHAIN_A_RPC : CHAIN_B_RPC;
    const destRpc = isAtoB ? CHAIN_B_RPC : CHAIN_A_RPC;

    const result = await sendInteropMessage(message, INTEROP_PRIVATE_KEY, addProgressStep, sourceRpc, destRpc);

    // Hide progress and show success
    document.getElementById("interop-progress").classList.add("hidden");
    document.getElementById("interop-success").classList.remove("hidden");

    // Fill in success details
    document.getElementById("interopTxHashSource").textContent = result.txHash;
    document.getElementById("interopBatchNumber").textContent = result.batchNumber;
    document.getElementById("interopMessageIndex").textContent = result.messageIndex;
    document.getElementById("interopMessageDirectionValue").textContent = isAtoB ? "Chain A → Chain B" : "Chain B → Chain A";

    console.log("✅ Interop verification successful!");
  } catch (error) {
    console.error("Interop failed:", error);
    document.getElementById("interop-progress").classList.add("hidden");
    document.getElementById("interop-error").textContent = `${text.interopFailed} ${error.message}`;
    document.getElementById("interop-error").classList.remove("hidden");
  } finally {
    btn.disabled = false;
    btn.textContent = text.sendMessage;
  }
}

// Token transfer functionality
// Can be set via VITE_TOKEN_ADDRESS environment variable (set automatically by deploy-usd.sh)
const TOKEN_ADDRESS = import.meta.env?.VITE_TOKEN_ADDRESS || "0xe441CF0795aF14DdB9f7984Da85CD36DB1B8790d";

// Update HTML with token address
if (typeof document !== "undefined") {
  document.addEventListener("DOMContentLoaded", () => {
    const tokenAddressEl = document.getElementById("tokenAddress");
    if (tokenAddressEl && TOKEN_ADDRESS) {
      tokenAddressEl.textContent = TOKEN_ADDRESS;
    }
  });
}

// Refresh token balances
async function handleRefreshTokenBalances() {
  const btn = document.getElementById("refreshTokenBalancesBtn");
  const text = translatedText();

  // Don't disable button if it doesn't exist (called from tab switch)
  if (btn) {
    btn.disabled = true;
    btn.textContent = text.refreshing;
  }

  try {
    console.log("🔄 Refreshing token balances...");

    // Create providers using ethers (for token-interop compatibility)
    const { ethers } = await import("ethers");
    const providerA = new ethers.JsonRpcProvider(CHAIN_A_RPC);
    const providerB = new ethers.JsonRpcProvider(CHAIN_B_RPC);

    // Get token info from Chain A
    const tokenInfo = await getTokenInfo(TOKEN_ADDRESS, providerA);
    document.getElementById("tokenName").textContent = tokenInfo.name;
    document.getElementById("tokenSymbol").textContent = tokenInfo.symbol;
    document.getElementById("tokenSymbolA").textContent = tokenInfo.symbol;
    document.getElementById("tokenSymbolB").textContent = tokenInfo.symbol;

    console.log("Token info:", tokenInfo);

    // Get wallet address (use the interop private key)
    const wallet = new ethers.Wallet(INTEROP_PRIVATE_KEY, providerA);
    const walletAddress = wallet.address;

    // Get balance on Chain A
    const balanceA = await getTokenBalance(TOKEN_ADDRESS, walletAddress, providerA);
    const formattedBalanceA = ethers.formatUnits(balanceA, tokenInfo.decimals);
    document.getElementById("tokenBalanceA").textContent = formattedBalanceA;

    console.log("Balance on Chain A:", formattedBalanceA, tokenInfo.symbol);

    // Get wrapped token address on Chain B
    const chainAId = (await providerA.getNetwork()).chainId;
    const wrappedTokenAddress = await getWrappedTokenAddress(TOKEN_ADDRESS, chainAId, providerB);

    if (wrappedTokenAddress) {
      const balanceB = await getTokenBalance(wrappedTokenAddress, walletAddress, providerB);
      const formattedBalanceB = ethers.formatUnits(balanceB, tokenInfo.decimals);
      document.getElementById("tokenBalanceB").textContent = formattedBalanceB;
      console.log("Balance on Chain B:", formattedBalanceB, tokenInfo.symbol);
    } else {
      document.getElementById("tokenBalanceB").textContent = `0 (${text.notBridgedYet})`;
      console.log("Token not yet bridged to Chain B");
    }

    console.log("✅ Token balances refreshed");
    // Hide warning on success
    document.getElementById("interop-connection-warning").classList.add("hidden");
    enableInteropForms();
  } catch (error) {
    console.error("Failed to refresh token balances:", error);
    // Show warning instead of alert
    showInteropWarning(error.message);
    disableInteropForms();
    // Only alert if triggered manually by button
    if (btn) {
      // Don't alert, warning is already shown
    }
  } finally {
    if (btn) {
      btn.disabled = false;
      btn.textContent = text.refreshTokenBalances;
    }
  }
}

// Show warning for interop connection issues
function showInteropWarning(message) {
  const warning = document.getElementById("interop-connection-warning");
  const messageEl = document.getElementById("interop-warning-message");
  messageEl.textContent = message;
  warning.classList.remove("hidden");
}

// Disable interop forms
function disableInteropForms() {
  document.getElementById("interopSendBtn").disabled = true;
  document.getElementById("tokenTransferBtn").disabled = true;
  document.getElementById("refreshTokenBalancesBtn").disabled = true;
  document.getElementById("interopMessage").disabled = true;
  document.getElementById("interopMessageDirection").disabled = true;
  document.getElementById("tokenTransferAmount").disabled = true;
  document.getElementById("tokenTransferDirection").disabled = true;
}

// Enable interop forms
function enableInteropForms() {
  document.getElementById("interopSendBtn").disabled = false;
  document.getElementById("tokenTransferBtn").disabled = false;
  document.getElementById("refreshTokenBalancesBtn").disabled = false;
  document.getElementById("interopMessage").disabled = false;
  document.getElementById("interopMessageDirection").disabled = false;
  document.getElementById("tokenTransferAmount").disabled = false;
  document.getElementById("tokenTransferDirection").disabled = false;
}

// Export to window for tab switch to access
window.handleRefreshTokenBalances = handleRefreshTokenBalances;

// Transfer tokens
async function handleTokenTransfer() {
  const btn = document.getElementById("tokenTransferBtn");
  const amountInput = document.getElementById("tokenTransferAmount");
  const directionSelect = document.getElementById("tokenTransferDirection");
  const amount = amountInput.value;
  const direction = directionSelect.value;
  const text = translatedText();

  if (!amount || parseFloat(amount) <= 0) {
    alert(text.amountError);
    return;
  }

  // Hide previous results
  document.getElementById("token-transfer-success").classList.add("hidden");
  document.getElementById("token-transfer-error").classList.add("hidden");
  document.getElementById("token-transfer-progress").classList.remove("hidden");

  btn.disabled = true;
  btn.textContent = text.transferring;

  const progressSteps = document.getElementById("tokenTransferProgressSteps");
  progressSteps.innerHTML = "";

  const addProgressStep = (step) => {
    const stepEl = document.createElement("div");
    stepEl.textContent = step;
    stepEl.style.padding = "4px 0";
    stepEl.style.fontSize = "13px";
    progressSteps.appendChild(stepEl);
  };

  try {
    console.log("🌉 Starting token transfer...");

    // Create providers using ethers
    const { ethers } = await import("ethers");
    const providerA = new ethers.JsonRpcProvider(CHAIN_A_RPC);
    const providerB = new ethers.JsonRpcProvider(CHAIN_B_RPC);

    // Determine source and destination based on direction
    const isAtoB = direction === "a-to-b";
    const sourceProvider = isAtoB ? providerA : providerB;
    const destProvider = isAtoB ? providerB : providerA;
    const sourceTokenAddress = TOKEN_ADDRESS; // Always the original token on Chain A

    // Get token info to parse amount with correct decimals
    const tokenInfo = await getTokenInfo(TOKEN_ADDRESS, providerA);
    const amountBigInt = ethers.parseUnits(amount, tokenInfo.decimals);

    // Get recipient address (same wallet)
    const wallet = new ethers.Wallet(INTEROP_PRIVATE_KEY, sourceProvider);
    const recipientAddress = wallet.address;

    // For B→A, we need to get the wrapped token address on Chain B
    let transferTokenAddress = sourceTokenAddress;
    const chainAId = (await providerA.getNetwork()).chainId;

    if (!isAtoB) {
      const wrappedTokenAddress = await getWrappedTokenAddress(TOKEN_ADDRESS, chainAId, providerB);
      if (!wrappedTokenAddress) {
        throw new Error("Token not yet bridged to Chain B. Please transfer from A to B first.");
      }
      transferTokenAddress = wrappedTokenAddress;
    }

    const result = await transferTokensInterop(
      transferTokenAddress,
      amountBigInt,
      recipientAddress,
      INTEROP_PRIVATE_KEY,
      sourceProvider,
      destProvider,
      TOKEN_ADDRESS, // originalTokenAddress - always Chain A token
      chainAId, // originalChainId - always Chain A ID
      addProgressStep,
    );

    // Hide progress and show success
    document.getElementById("token-transfer-progress").classList.add("hidden");
    document.getElementById("token-transfer-success").classList.remove("hidden");

    // Fill in success details
    document.getElementById("tokenTransferTxHashSource").textContent = result.sendTxHash;
    document.getElementById("tokenTransferTxHashDest").textContent = result.executeTxHash;
    document.getElementById("tokenTransferAmountValue").textContent = amount;
    document.getElementById("tokenTransferSymbol").textContent = tokenInfo.symbol;
    document.getElementById("tokenTransferDirectionValue").textContent = isAtoB ? text.aToB : text.bToA;

    console.log("✅ Token transfer successful!");

    // Refresh balances after 2 seconds
    setTimeout(() => {
      handleRefreshTokenBalances();
    }, 2000);
  } catch (error) {
    console.error("Token transfer failed:", error);
    document.getElementById("token-transfer-progress").classList.add("hidden");
    document.getElementById("token-transfer-error").textContent = `${text.tokenTransferFailed} ${error.message}`;
    document.getElementById("token-transfer-error").classList.remove("hidden");
  } finally {
    btn.disabled = false;
    btn.textContent = text.transferTokens;
  }
}
