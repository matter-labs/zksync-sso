import { type Address, createWalletClient, http, parseEther, publicActions, toHex } from "viem";
import { generatePrivateKey, privateKeyToAccount, privateKeyToAddress } from "viem/accounts";
import { deployAccount } from "zksync-sso/client";
import { registerNewPasskey } from "zksync-sso/client/passkey";
import type { SessionConfig } from "zksync-sso/utils";

export const useAccountCreate = (_chainId: MaybeRef<SupportedChainId>) => {
  const chainId = toRef(_chainId);
  const { login } = useAccountStore();
  const { getThrowAwayClient } = useClientStore();

  const getRichClient = () => {
    const chain = supportedChains.find((c) => c.id === chainId.value);
    if (!chain) throw new Error("Unsupported chain when funding");
    const client = createWalletClient({
      account: privateKeyToAccount("0xac1e735be8536c6534bb4f17f06f6afc73b2b5ba84ac2cfb12f7461b20c0bbe3"),
      chain,
      transport: http(),
    }).extend(publicActions);
    return client;
  };
  const fundAccount = async (address: Address) => {
    const client = getRichClient();
    const transactionHash = await client.sendTransaction({
      to: address,
      value: parseEther("0.5"),
      maxPriorityFeePerGas: 0n,
    });
    const receipt = await client.waitForTransactionReceipt({ hash: transactionHash });
    if (receipt.status !== "success") {
      throw new Error("Failed to fund account");
    }
  };

  const mintAAToken = async (address: Address) => {
    const client = getRichClient();
    const transactionHash = await client.writeContract({
      address: "0xd4567AA4Fd1B32A16c16CBFF9D9a69e51CF72293",
      abi: [
        {
          inputs: [
            {
              internalType: "address",
              name: "to",
              type: "address",
            },
            {
              internalType: "uint256",
              name: "amount",
              type: "uint256",
            },
          ],
          name: "mint",
          outputs: [],
          stateMutability: "nonpayable",
          type: "function",
        },
      ] as const,
      functionName: "mint",
      args: [address, parseEther("10")],
    });
    const receipt = await client.waitForTransactionReceipt({ hash: transactionHash });
    if (receipt.status !== "success") {
      throw new Error("Failed to fund account");
    }
  };

  const { inProgress: registerInProgress, error: createAccountError, execute: createAccount } = useAsync(async (session?: Omit<SessionConfig, "signer">) => {
    // Format passkey display name similar to "ZKsync SSO 11/11/2024 01:46 PM"
    let name = `ZKsync SSO ${(new Date()).toLocaleDateString("en-US")}`;
    name += ` ${(new Date()).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}`;

    const {
      credentialPublicKey,
      credentialId,
    } = await registerNewPasskey({
      userName: name,
      userDisplayName: name,
    });

    let sessionData: SessionConfig | undefined;
    const sessionKey = generatePrivateKey();
    const signer = privateKeyToAddress(sessionKey);
    if (session) {
      sessionData = {
        ...session,
        signer: signer,
      };
    }

    const deployerClient = getThrowAwayClient({ chainId: chainId.value });

    console.log("Deploying account");
    const deployedAccount = await deployAccount(deployerClient, {
      credentialPublicKey,
      uniqueAccountId: credentialId,
      contracts: contractsByChain[chainId.value],
      paymasterAddress: contractsByChain[chainId.value].accountPaymaster,
      initialSession: sessionData || undefined,
    });
    await retry(async () => fundAccount(deployedAccount.address), { delay: 1000 });
    await mintAAToken(deployedAccount.address);

    login({
      username: credentialId,
      address: deployedAccount.address,
      passkey: toHex(credentialPublicKey),
    });

    return {
      address: deployedAccount.address,
      chainId: chainId.value,
      sessionKey: session ? sessionKey : undefined,
      signer,
      sessionConfig: sessionData,
    };
  });

  return {
    registerInProgress,
    createAccount,
    createAccountError,
  };
};
