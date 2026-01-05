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
  getShadowAccount,
  createAaveDepositBundle,
  createAaveWithdrawBundle,
  getAaveBalance,
} from "./aave-utils.js";

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
const isBrowser = typeof window !== "undefined";
const shouldUseLocalRpcProxy = isBrowser && (window.location.hostname === "localhost" || window.location.hostname === "127.0.0.1");
const ZKSYNC_OS_RPC_URL = shouldUseLocalRpcProxy ? LOCAL_RPC_PROXY_URL : DEFAULT_ZKSYNC_OS_RPC_URL;

// Deployer private key (for deploying accounts)
// const DEPLOYER_PRIVATE_KEY = "0xef506537558847aa991149381c4fedee8fe1252cf868986ac1692336530ec85c";
const DEPLOYER_PRIVATE_KEY = "0x1cfcab2cf5ad255cb3387f7fdca2651a61377b334a2a3daa4af86eb476369105";

// Contract addresses on ZKsync OS testnet
const CONTRACTS = {
  p256Verifier: "0xD65900405073D912215bC8dEb811dFFD72263065", // P256VerifierNoPrecompile - Pure Solidity!
  passkey: "0xc66A20c63606f221D4d5A39147E3bf3635DD7a39", // WebAuthnValidator (uses pure Solidity verifier)
  session: "0xbd2608f3512A3a163394fbAB99a286E151Bf30be", // SessionKeyValidator
  accountFactory: "0xF52708DE29453BBfd27AA8fC7b4bc7EF87E05892", // MSAFactory
  entryPoint: "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108",
  bundlerUrl: "http://localhost:4337", // Local bundler for better debugging
  oidcKeyRegistry: "0x0000000000000000000000000000000000000000",
};
// State
let accountAddress = null;
let passkeyCredentials = null;
let publicClient = null;
let sepoliaClient = null;
let shadowAccount = null;

// LocalStorage keys
const STORAGE_KEY_PASSKEY = "zksync_sso_passkey";
const STORAGE_KEY_ACCOUNT = "zksync_sso_account";

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

// Initialize
document.addEventListener("DOMContentLoaded", () => {
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

  setupEventListeners();
  loadExistingPasskey();
});

function setupEventListeners() {
  document.getElementById("createPasskeyBtn").addEventListener("click", handleCreatePasskey);
  document.getElementById("deployAccountBtn").addEventListener("click", handleDeployAccount);
  document.getElementById("refreshBalanceBtn").addEventListener("click", handleRefreshBalance);
  document.getElementById("transferBtn").addEventListener("click", handleTransfer);
  document.getElementById("resetPasskeyBtn").addEventListener("click", handleResetPasskey);
  document.getElementById("aaveDepositBtn").addEventListener("click", handleAaveDeposit);
  document.getElementById("aaveWithdrawBtn").addEventListener("click", handleAaveWithdraw);
  document.getElementById("refreshAaveBalanceBtn").addEventListener("click", refreshAaveBalance);
}

// Load existing passkey from localStorage
function loadExistingPasskey() {
  const savedPasskey = localStorage.getItem(STORAGE_KEY_PASSKEY);
  const savedAccount = localStorage.getItem(STORAGE_KEY_ACCOUNT);

  if (savedPasskey) {
    passkeyCredentials = JSON.parse(savedPasskey);

    // Show passkey success
    document.getElementById("passkey-input").classList.add("hidden");
    document.getElementById("passkey-success").classList.remove("hidden");
    document.getElementById("credentialIdDisplay").textContent = passkeyCredentials.credentialId;

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

      // Enable transfer
      document.getElementById("transferBtn").disabled = false;

      handleRefreshBalance();
      initializeAaveSection();

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
    location.reload();
  }
}

// Step 1: Create Passkey
async function handleCreatePasskey() {
  const userName = document.getElementById("userName").value.trim();
  if (!userName) {
    alert("Please enter your name");
    return;
  }

  const btn = document.getElementById("createPasskeyBtn");
  btn.disabled = true;
  btn.textContent = "Creating...";

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

    // Enable deploy button
    document.getElementById("deployAccountBtn").disabled = false;
  } catch (error) {
    console.error("Passkey creation failed:", error);
    document.getElementById("passkey-error").textContent = "Failed to create passkey: " + error.message;
    document.getElementById("passkey-error").classList.remove("hidden");
  } finally {
    btn.disabled = false;
    btn.textContent = "Create Passkey";
  }
}

// Step 2: Deploy Account
async function handleDeployAccount() {
  const btn = document.getElementById("deployAccountBtn");
  btn.disabled = true;
  btn.textContent = "Deploying...";

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

    // Enable transfer
    document.getElementById("transferBtn").disabled = false;

    handleRefreshBalance();
    initializeAaveSection();
  } catch (error) {
    console.error("Deployment failed:", error);
    document.getElementById("deploy-error").textContent = "Deployment failed: " + error.message;
    document.getElementById("deploy-error").classList.remove("hidden");
  } finally {
    btn.disabled = false;
    btn.textContent = "Deploy Account";
  }
}

// Step 3: Refresh Balance
async function handleRefreshBalance() {
  if (!accountAddress) return;

  const btn = document.getElementById("refreshBalanceBtn");
  btn.disabled = true;
  btn.textContent = "Refreshing...";

  try {
    const balance = await publicClient.getBalance({
      address: accountAddress,
    });

    const balanceInEth = formatEther(balance);
    document.getElementById("balanceDisplay").textContent = `${balanceInEth} ETH`;
    console.log(`💰 Balance: ${balanceInEth} ETH`);
  } catch (error) {
    console.error("Balance fetch failed:", error);
  } finally {
    btn.disabled = false;
    btn.textContent = "Refresh Balance";
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

// Step 4: Transfer ETH using ERC-4337 bundler
async function handleTransfer() {
  const recipient = document.getElementById("recipientAddress").value.trim();
  const amount = document.getElementById("transferAmount").value.trim();

  if (!recipient || !amount) {
    alert("Please enter recipient address and amount");
    return;
  }

  const btn = document.getElementById("transferBtn");
  btn.disabled = true;
  btn.textContent = "Transferring...";

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
    document.getElementById("transfer-error").textContent = "Transfer failed: " + error.message;
    document.getElementById("transfer-error").classList.remove("hidden");
  } finally {
    btn.disabled = false;
    btn.textContent = "Send ETH";
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

    console.log("✅ Aave section initialized");
  } catch (error) {
    console.error("Failed to initialize Aave section:", error);
  }
}

// Refresh Aave balance (aToken balance on L1)
async function refreshAaveBalance() {
  if (!shadowAccount) return;

  const btn = document.getElementById("refreshAaveBalanceBtn");
  if (btn) {
    btn.disabled = true;
    btn.textContent = "Refreshing...";
  }

  try {
    console.log("🔵 Checking Aave balance on L1 Sepolia...");

    const balance = await getAaveBalance(sepoliaClient, shadowAccount);
    console.log(`Aave balance: ${formatEther(balance)} aETH`);

    // Update UI
    document.getElementById("aaveBalanceDisplay").textContent = formatEther(balance) + " aETH";
  } catch (error) {
    console.error("Failed to refresh Aave balance:", error);
    document.getElementById("aaveBalanceDisplay").textContent = "Error loading balance";
  } finally {
    if (btn) {
      btn.disabled = false;
      btn.textContent = "Refresh Balance";
    }
  }
}

// Handle Aave deposit
async function handleAaveDeposit() {
  if (!accountAddress || !shadowAccount) return;

  const btn = document.getElementById("aaveDepositBtn");
  btn.disabled = true;
  btn.textContent = "Depositing...";

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
    const bundle = await createAaveDepositBundle(amountWei, shadowAccount);
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

      // Update UI
      document.getElementById("aave-deposit-success").classList.remove("hidden");
      document.getElementById("aaveDepositTxHash").textContent = receipt.receipt.transactionHash;

      // Clear form
      document.getElementById("aaveDepositAmount").value = "";

      // Note: Balance will update after L1 finalization (~15 min)
      console.log("💡 Note: Aave balance will update after L2-to-L1 finalization (~15 minutes)");
    } else {
      throw new Error("Transaction failed");
    }
  } catch (error) {
    console.error("Aave deposit failed:", error);
    document.getElementById("aave-deposit-error").textContent = "Deposit failed: " + error.message;
    document.getElementById("aave-deposit-error").classList.remove("hidden");
  } finally {
    btn.disabled = false;
    btn.textContent = "Deposit to Aave";
  }
}

// Handle Aave withdraw
async function handleAaveWithdraw() {
  if (!accountAddress || !shadowAccount) return;

  const btn = document.getElementById("aaveWithdrawBtn");
  btn.disabled = true;
  btn.textContent = "Withdrawing...";

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

      // Update UI
      document.getElementById("aave-withdraw-success").classList.remove("hidden");
      document.getElementById("aaveWithdrawTxHash").textContent = receipt.receipt.transactionHash;

      // Clear form
      document.getElementById("aaveWithdrawAmount").value = "";

      // Note: Balance will update after L1 finalization (~15 min)
      console.log("💡 Note: Aave balance will update after L2-to-L1 finalization (~15 minutes)");
    } else {
      throw new Error("Transaction failed");
    }
  } catch (error) {
    console.error("Aave withdrawal failed:", error);
    document.getElementById("aave-withdraw-error").textContent = "Withdrawal failed: " + error.message;
    document.getElementById("aave-withdraw-error").classList.remove("hidden");
  } finally {
    btn.disabled = false;
    btn.textContent = "Withdraw from Aave";
  }
}
