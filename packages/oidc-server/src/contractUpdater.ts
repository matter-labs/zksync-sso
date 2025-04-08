import { Contract, Provider, types, Wallet } from "zksync-ethers";
import { CircomBigInt } from "zksync-sso-circuits";

import { abi } from "./abi.js";
import { env } from "./env.js";
import type { BaseKey, Key } from "./types.js";

export class ContractUpdater {
  private wallet: Wallet;
  private provider: Provider;
  private contract: Contract;
  private issHashes = new Map<string, string>();

  constructor() {
    if (env.RPC_URL) {
      this.provider = new Provider(env.RPC_URL);
    } else if (env.NETWORK) {
      this.provider = Provider.getDefaultProvider(this.getNetwork());
    } else {
      throw new Error("Either RPC_URL or NETWORK must be set");
    }
    this.wallet = new Wallet(env.ADMIN_PRIVATE_KEY, this.provider);

    this.contract = new Contract(
      env.CONTRACT_ADDRESS,
      abi,
      this.wallet,
    );
  }

  public async updateContract(iss: string, keys: BaseKey[]): Promise<void> {
    console.log(`Updating contract for issuer: ${iss}`);

    const issHash = await this.getIssHash(iss);
    const newKeys = await this.getNewKeys(issHash, keys);

    if (newKeys.length === 0) {
      console.log("No new keys to add.");
      return;
    }

    try {
      const tx = await this.contract.addKeys(newKeys);
      console.log(`Transaction sent: ${tx.hash}`);
      await tx.wait();
      console.log("Transaction confirmed!");
    } catch (error) {
      console.error("Error updating contract:", error);
    }
  }

  private async getNewKeys(issHash: string, keys: BaseKey[]): Promise<Key[]> {
    const promises = keys.map((key) => {
      const n = CircomBigInt.fromBase64(key.n).serialize();

      return this.contract.getKey(issHash, key.kid).then(
        () => null,
        () => ({
          ...key,
          n,
          issHash,
        }),
      );
    },
    );

    const results = await Promise.all(promises);
    return results.filter((key): key is Key => key !== null);
  }

  private getNetwork(): types.Network {
    if (!env.NETWORK) {
      throw new Error("NETWORK is not set in config");
    }

    const networkMap: Record<string, types.Network> = {
      mainnet: types.Network.Mainnet,
      sepolia: types.Network.Sepolia,
      localhost: types.Network.Localhost,
    };

    const network = networkMap[env.NETWORK.toLowerCase()];

    if (!network) {
      throw new Error(`Unknown or unsupported network: ${env.NETWORK}`);
    }

    return network;
  }

  private async getIssHash(iss: string): Promise<string> {
    let issHash = this.issHashes.get(iss);
    if (!issHash) {
      issHash = await this.contract.hashIssuer.staticCall(iss);
      if (!issHash) {
        throw new Error("Failed to get issuer hash");
      }
      this.issHashes.set(iss, issHash);
    }
    return issHash;
  }
}
