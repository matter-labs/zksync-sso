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
      url: "http://localhost:8545",
      chainId: 31337,
    },
  },
  solidity: {
    version: "0.8.23",
  },
};

export default config;
