import "@matterlabs/hardhat-zksync";

import { HardhatUserConfig } from "hardhat/config";

const config: HardhatUserConfig = {
  defaultNetwork: "zkSyncSepoliaTestnet",
  networks: {
    zkSyncSepoliaTestnet: {
      url: "https://sepolia.era.zksync.dev",
      ethNetwork: "sepolia",
      zksync: true,
      verifyURL: "https://explorer.sepolia.era.zksync.dev/contract_verification",
    },
    zkSyncMainnet: {
      url: "https://mainnet.era.zksync.io",
      ethNetwork: "mainnet",
      zksync: true,
      verifyURL: "https://zksync2-mainnet-explorer.zksync.io/contract_verification",
    },
    dockerizedNode: {
      url: "http://localhost:3050",
      ethNetwork: "http://localhost:8545",
      zksync: true,
    },
    inMemoryNode: {
      url: "http://127.0.0.1:8011",
      ethNetwork: "localhost", // in-memory node doesn't support eth node; removing this line will cause an error
      zksync: true,
    },
    hardhat: {
      zksync: true,
    },
    localhost: {
      url: "http://127.0.0.1:8545",
      // This network uses a standard local Ethereum node (e.g., Anvil or Hardhat Network) that does not support zkSync-specific features.
      // zkSync is disabled here to allow minimal ERC721 deployment and testing without zkSync extensions.
      zksync: false,
    },
  },
  zksolc: {
    version: "latest",
    settings: {
      // enable viaIR for complex system contract compilation when needed
      optimizer: {
        enabled: true,
        runs: 200,
      },
      viaIR: true,
    },
  },
  solidity: {
    version: "0.8.17",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,
      },
      viaIR: true,
    },
  },
};

export default config;
