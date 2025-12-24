import { createWalletClient, type Hex, http, publicActions } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { zksync, zksyncInMemoryNode, zksyncSepoliaTestnet } from "viem/chains";
import { OidcKeyRegistryAbi } from "zksync-sso/abi";
import { CircomBigInt } from "zksync-sso-circuits";

import { type BaseKey, type Key, keySchema, type ValidNetworks } from "./types.js";

export class ContractUpdater {
  private issHashes = new Map<string, Hex>();
  private rpcUrl: string;
  private contractAddress: Hex;
  private privKey: Hex;
  private network: typeof zksyncInMemoryNode | typeof zksync | typeof zksyncSepoliaTestnet;

  constructor(contractAddress: Hex, rpcUrl: string, privKey: Hex, networkName: ValidNetworks) {
    this.rpcUrl = rpcUrl;
    this.privKey = privKey;
    this.contractAddress = contractAddress;
    this.network = networkName === "local"
      ? zksyncInMemoryNode
      : networkName === "sepolia"
        ? zksyncSepoliaTestnet
        : zksync;
  }

  private client() {
    return createWalletClient({
      chain: this.network,
      account: privateKeyToAccount(this.privKey),
      transport: http(this.rpcUrl),
    }).extend(publicActions);
  }

  public async updateContract(iss: string, keys: BaseKey[]): Promise<void> {
    const client = this.client();
    console.log(`Adding oidc keys for: ${iss}`);

    const issHash = await this.getIssHash(iss);
    const newKeys = await this.filterKeys(issHash, keys);

    if (newKeys.length === 0) {
      console.log("No new keys to add.");
      return;
    }

    try {
      const txHash = await client.writeContract({
        abi: OidcKeyRegistryAbi,
        functionName: "addKeys",
        address: this.contractAddress,
        args: [newKeys],
      });
      console.log(`Transaction sent: ${txHash}`);
      await client.waitForTransactionReceipt({ hash: txHash });
      console.log("Transaction confirmed!");
    } catch (error) {
      console.error("Error updating contract:", error);
    }
  }

  private async filterKeys(issHash: Hex, keysFromProvider: BaseKey[]): Promise<Key[]> {
    const client = this.client();

    const rawContractKeys = await client.readContract({
      functionName: "getKeys",
      abi: OidcKeyRegistryAbi,
      address: this.contractAddress,
      args: [issHash],
    });

    const filteredKeys = keysFromProvider
      .filter((k) =>
        rawContractKeys.every((ck) => ck.kid !== k.kid),
      );

    return filteredKeys.map((bk) => keySchema.parse({
      issHash: issHash,
      kid: bk.kid,
      rsaModulus: CircomBigInt.fromBase64(bk.n).serialize()
        .map((bn) => BigInt(bn)),
    }));
  }

  private async getIssHash(iss: string): Promise<Hex> {
    const storedIssHash = this.issHashes.get(iss);
    if (!storedIssHash) {
      const client = this.client();
      const issHash = await client.readContract({
        address: this.contractAddress,
        abi: OidcKeyRegistryAbi,
        functionName: "hashIssuer",
        args: [iss],
      });
      this.issHashes.set(iss, issHash);
      return issHash;
    }
    return storedIssHash;
  }
}
