import "@matterlabs/hardhat-zksync";
import type { HardhatUserConfig } from "hardhat/config";

const config: HardhatUserConfig = {
  zksolc: {
    version: "latest",
    settings: {
      // Note: This must be false to call solc for compilation
      isSystem: false,
    },
  },
  defaultNetwork: "anvil",
  networks: {
    anvil: {
      url: "http://localhost:8546",
      chainId: 9,
    },
    zkSyncTestnet: {
      url: "http://localhost:8011",
      ethNetwork: "http://localhost:8545",
      zksync: true,
    },
  },
  solidity: {
    version: "0.8.23",
  },
};

export default config;
