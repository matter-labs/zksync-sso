import { encodeFunctionData } from "viem";

// Contract addresses on Sepolia (L1)
export const AAVE_CONTRACTS = {
  // L2 Contract (ZKSync OS Testnet)
  l2InteropCenter: "0xc64315efbdcD90B71B0687E37ea741DE0E6cEFac",

  // L1 Contracts (Sepolia)
  aavePool: "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951",
  aaveWeth: "0x387d311e47e80b498169e6fb51d3193167d89F7D", // Wrapped Token Gateway
  aaveWethToken: "0xC558DBdd856501FCd9aaF1E62eae57A9F0629a3c",
  aToken: "0x5b071b590a59395fE4025A0Ccc1FcC931AAc1830",
  ghoToken: "0xc4bF5CbDaBE595361438F8c6a187bDc330539c60",
};

const BASE_TOKEN_ADDRESS = "0x000000000000000000000000000000000000800A";

const L2_BASE_TOKEN_ABI = [
  {
    type: "function",
    name: "withdraw",
    inputs: [{ name: "l1Receiver", type: "address" }],
    outputs: [],
    stateMutability: "payable",
  },
];

// ZKSync L1 Bridge address (Sepolia)
const BRIDGEHUB_ADDRESS = "0xc4FD2580C3487bba18D63f50301020132342fdbD";

// ZKSync Chain ID (ZKSync OS Testnet)
const L2_CHAIN_ID = 8022833n;

// L2 gas parameters for bridging
const L2_GAS_LIMIT = 300000n;
const L2_GAS_PER_PUBDATA = 800n;

/**
 * Get the shadow account address for an L2 address
 * This is the L1 address that will perform actions on behalf of the L2 account
 */
export async function getShadowAccount(publicClient, l2Address) {
  const L2_INTEROP_CENTER_ABI = (await import("./abis/L2InteropCenter.json")).default;

  const shadowAccount = await publicClient.readContract({
    address: AAVE_CONTRACTS.l2InteropCenter,
    abi: L2_INTEROP_CENTER_ABI.abi,
    functionName: "l1ShadowAccount",
    args: [l2Address],
  });

  return shadowAccount;
}

/**
 * Create a bundle to deposit ETH into Aave on L1
 * @param {bigint} amount - Amount of ETH to deposit (in wei)
 * @param {string} shadowAccount - L1 shadow account address
 * @returns {Object} Bundle operation for L2InteropCenter
 */
export async function createAaveDepositBundle(amount, shadowAccount) {
  const I_WRAPPED_TOKEN_ABI = (await import("./abis/IWrappedTokenGatewayV3.json")).default;
  const L2_INTEROP_CENTER_ABI = (await import("./abis/L2InteropCenter.json")).default;

  const withdrawData = encodeFunctionData({
    abi: L2_BASE_TOKEN_ABI,
    functionName: "withdraw",
    args: [shadowAccount],
  });

  // Encode the depositETH function call
  // This will be executed by the shadow account on L1
  const depositETHData = encodeFunctionData({
    abi: I_WRAPPED_TOKEN_ABI.abi,
    functionName: "depositETH",
    args: [
      AAVE_CONTRACTS.aavePool, // Aave pool address
      shadowAccount, // On behalf of (receives aTokens)
      0, // Referral code
    ],
  });

  // Create the operation bundle
  const ops = [
    {
      target: AAVE_CONTRACTS.aaveWeth, // WETH Gateway contract
      value: amount, // ETH to deposit
      data: depositETHData, // Function call data
    },
  ];

  return {
    withdrawCall: {
      to: BASE_TOKEN_ADDRESS,
      value: amount,
      data: withdrawData,
    },
    bundle: {
      address: AAVE_CONTRACTS.l2InteropCenter,
      abi: L2_INTEROP_CENTER_ABI.abi,
      functionName: "sendBundleToL1",
      args: [ops],
      value: 0n, // No value sent to L2InteropCenter itself
    },
  };
}

/**
 * Create a bundle to withdraw ETH from Aave on L1
 * @param {bigint} amount - Amount of aETH to withdraw (in wei)
 * @param {string} shadowAccount - L1 shadow account address
 * @param {string} l2Receiver - L2 address to receive the ETH
 * @param {Object} l1Client - L1 public client for gas estimation
 * @returns {Object} Bundle operation for L2InteropCenter
 */
export async function createAaveWithdrawBundle(amount, shadowAccount, l2Receiver, l1Client) {
  const AAVE_POOL_ABI = (await import("./abis/IPool.json")).default;
  const WETH_ABI = (await import("./abis/IWETH.json")).default;
  const L1_BRIDGEHUB_ABI = (await import("../aave-interop-demo/utils/abis/IL1Bridgehub.json")).default;
  const L2_INTEROP_CENTER_ABI = (await import("./abis/L2InteropCenter.json")).default;

  // Calculate mintValue for L2 gas
  const gasPrice = await l1Client.getGasPrice();
  const baseCost = await l1Client.readContract({
    address: BRIDGEHUB_ADDRESS,
    abi: L1_BRIDGEHUB_ABI.abi,
    functionName: "l2TransactionBaseCost",
    args: [L2_CHAIN_ID, gasPrice, L2_GAS_LIMIT, L2_GAS_PER_PUBDATA],
  });
  const mintValue = baseCost + (baseCost * 20n) / 100n; // 20% buffer

  // Step 1: Withdraw from Aave Pool (gets WETH)
  const withdrawData = encodeFunctionData({
    abi: AAVE_POOL_ABI.abi,
    functionName: "withdraw",
    args: [
      AAVE_CONTRACTS.aaveWethToken, // WETH asset address
      amount, // Amount to withdraw
      shadowAccount, // Recipient (gets WETH)
    ],
  });

  // Step 2: Unwrap WETH to ETH
  const unwrapData = encodeFunctionData({
    abi: WETH_ABI.abi,
    functionName: "withdraw",
    args: [amount], // Amount of WETH to unwrap
  });

  // Step 3: Bridge ETH back to L2
  // mintValue = base fee; total value sent = l2Value + mintValue
  const totalBridgeValue = mintValue + amount;

  const bridgeData = encodeFunctionData({
    abi: L1_BRIDGEHUB_ABI.abi,
    functionName: "requestL2TransactionDirect",
    args: [{
      chainId: L2_CHAIN_ID,
      mintValue: totalBridgeValue, // Exact fee + l2Value
      l2Contract: l2Receiver,
      l2Value: amount, // Amount to send to L2 contract as msg.value
      l2Calldata: "0x",
      l2GasLimit: L2_GAS_LIMIT,
      l2GasPerPubdataByteLimit: L2_GAS_PER_PUBDATA,
      factoryDeps: [],
      refundRecipient: "0x18ac402d33706c303cD559FA6B2F2f14Fae75307",
    }],
  });

  // Create the operation bundle
  const ops = [
    {
      target: AAVE_CONTRACTS.aavePool, // Aave Pool contract
      value: 0n,
      data: withdrawData, // Withdraw WETH from pool
    },
    {
      target: AAVE_CONTRACTS.aaveWethToken, // WETH contract
      value: 0n,
      data: unwrapData, // Unwrap WETH to ETH
    },
    {
      target: BRIDGEHUB_ADDRESS,
      value: totalBridgeValue, // Total ETH for bridge (gas + amount)
      data: bridgeData,
    },
  ];

  return {
    address: AAVE_CONTRACTS.l2InteropCenter,
    abi: L2_INTEROP_CENTER_ABI.abi,
    functionName: "sendBundleToL1",
    args: [ops],
    value: 0n,
  };
}

/**
 * Get L2 withdrawal parameters for sending ETH to shadow account
 * This is the first step - withdraw from L2 to L1 shadow account
 */
export function getL2WithdrawalParams(amount, shadowAccount) {
  return {
    token: "0x0000000000000000000000000000000000000000", // ETH
    amount,
    to: shadowAccount,
  };
}

/**
 * Check Aave deposit balance (aTokens) on L1
 */
export async function getAaveBalance(publicClient, shadowAccount) {
  const IERC20_ABI = (await import("./abis/IERC20.json")).default;

  try {
    const balance = await publicClient.readContract({
      address: AAVE_CONTRACTS.aToken,
      abi: IERC20_ABI.abi,
      functionName: "balanceOf",
      args: [shadowAccount],
    });

    return balance;
  } catch (error) {
    console.error("Error fetching Aave balance:", error);
    return 0n;
  }
}

/**
 * Encode the full L2-to-L1 Aave deposit flow as calldata for UserOperation
 * This combines:
 * 1. L2 withdrawal to shadow account
 * 2. Bundle execution to deposit into Aave
 */
export async function encodeAaveDepositCalldata(amount, shadowAccount) {
  const { bundle, withdrawCall } = await createAaveDepositBundle(amount, shadowAccount);

  return {
    withdrawCall,
    bundleCall: {
      to: AAVE_CONTRACTS.l2InteropCenter,
      value: 0n,
      data: encodeFunctionData({
        abi: bundle.abi,
        functionName: bundle.functionName,
        args: bundle.args,
      }),
    },
  };
}
