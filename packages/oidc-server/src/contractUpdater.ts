import type { Hex } from "viem";

import type { BaseKey, ValidNetworks } from "./types.js";

// TODO: Restore the on-chain OIDC key registry updater once OIDC support is
// reintroduced for the active zksync-os stack. The auth-server OIDC UI is
// currently disabled, and there is no e2e coverage for this path.
export class ContractUpdater {
  constructor(
    contractAddress: Hex,
    rpcUrl: string,
    privKey: Hex,
    networkName: ValidNetworks,
  ) {
    void contractAddress;
    void rpcUrl;
    void privKey;
    void networkName;
  }

  public async updateContract(iss: string, keys: BaseKey[]): Promise<void> {
    console.warn(
      `Skipping OIDC key registry update for ${iss}. `
      + `OIDC is currently disabled for the active zksync-os flow.`,
    );
    console.warn(`Fetched ${keys.length} keys but did not submit an on-chain update.`);
  }
}
