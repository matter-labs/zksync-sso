import { Wallet } from "ethers";
import { Contract } from "ethers";
import { Provider, types } from "zksync-ethers";

import { abi } from "./abi";
import { config } from "./config";
import type { Key } from "./types";

export class ContractUpdater {
  private wallet: Wallet;
  private provider: Provider;
  private contract: Contract;
  private issHashes = new Map<string, string>();

  constructor() {
    if (config.RPC_URL) {
      this.provider = new Provider(config.RPC_URL);
    } else if (config.NETWORK) {
      this.provider = Provider.getDefaultProvider(this.getNetwork());
    } else {
      throw new Error("Either RPC_URL or NETWORK must be set");
    }
    this.wallet = new Wallet(config.ZKSYNC_PRIVATE_KEY, this.provider);

    this.contract = new Contract(
      config.CONTRACT_ADDRESS,
      abi,
      this.wallet,
    );
  }

  public async updateContract(iss: string, keys: Key[]): Promise<void> {
    console.log(`Updating contract for issuer: ${iss}`);

    const issHash = await this.getIssHash(iss);
    const newKeys = await this.getNewKeys(issHash, keys);

    if (newKeys.length === 0) {
      console.log("No new keys to add.");
      return;
    }

    try {
      const tx = await this.contract.setKeys(issHash, newKeys);
      console.log(`Transaction sent: ${tx.hash}`);
      await tx.wait();
      console.log("Transaction confirmed!");
    } catch (error) {
      console.error("Error updating contract:", error);
    }
  }

  private async getNewKeys(issHash: string, keys: Key[]): Promise<Key[]> {
    const promises = keys.map((key) =>
      this.contract.getKey(issHash, key.kid).then(
        () => null,
        () => key,
      ),
    );

    const results = await Promise.all(promises);

    return results.filter((key): key is Key => key !== null);
  }

  private getNetwork(): types.Network {
    if (!config.NETWORK) {
      throw new Error("NETWORK is not set in config");
    }

    const networkMap: Record<string, types.Network> = {
      mainnet: types.Network.Mainnet,
      sepolia: types.Network.Sepolia,
      localhost: types.Network.Localhost,
    };

    const network = networkMap[config.NETWORK.toLowerCase()];

    if (!network) {
      throw new Error(`Unknown or unsupported network: ${config.NETWORK}`);
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
