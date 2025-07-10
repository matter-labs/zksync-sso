import { promises as fs } from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const version = process.env.INPUT_VERSION;

if (!version) {
  console.error("Error: INPUT_VERSION is required.");
  process.exit(1);
}

const packageJsonPath = path.join(__dirname, "package.json");

async function prepareWagmiConnectorPackage() {
  try {
    const packageJsonData = await fs.readFile(packageJsonPath, "utf8");
    const packageJson = JSON.parse(packageJsonData);

    // Create a minimal package.json for wagmi connector
    const wagmiConnectorPackage = {
      name: "zksync-sso-wagmi-connector",
      version: version,
      description: "ZKsync SSO Wagmi Connector - Lightweight connector for wagmi integration",
      license: packageJson.license,
      author: packageJson.author,
      repository: packageJson.repository,
      keywords: [
        "zksync",
        "wagmi",
        "connector",
        "web3",
        "ethereum",
        "wallet",
      ],
      type: "module",
      sideEffects: false,
      main: "./dist/_cjs/connector/index.js",
      module: "./dist/_esm/connector/index.js",
      types: "./dist/_types/connector/index.d.ts",
      typings: "./dist/_types/connector/index.d.ts",
      exports: {
        ".": {
          types: "./dist/_types/connector/index.d.ts",
          import: "./dist/_esm/connector/index.js",
          require: "./dist/_cjs/connector/index.js",
        },
        "./package.json": "./package.json",
      },
      files: [
        "dist/_cjs/connector/",
        "dist/_esm/connector/",
        "dist/_types/connector/",
        "dist/_cjs/client-auth-server/",
        "dist/_esm/client-auth-server/",
        "dist/_types/client-auth-server/",
        "dist/_cjs/client/",
        "dist/_esm/client/",
        "dist/_types/client/",
        "dist/_cjs/errors/",
        "dist/_esm/errors/",
        "dist/_types/errors/",
        "dist/_cjs/paymaster/",
        "dist/_esm/paymaster/",
        "dist/_types/paymaster/",
        "dist/_cjs/communicator/",
        "dist/_esm/communicator/",
        "dist/_types/communicator/",
        "dist/_cjs/utils/",
        "dist/_esm/utils/",
        "dist/_types/utils/",
        "dist/_cjs/abi/",
        "dist/_esm/abi/",
        "dist/_types/abi/",
        "dist/_cjs/types/",
        "dist/_esm/types/",
        "dist/_types/types/",
        "dist/_cjs/index.js",
        "dist/_esm/index.js",
        "dist/_types/index.d.ts",
        "!**/*.test.ts",
        "!**/*.test-d.ts",
        "!**/*.tsbuildinfo",
      ],
      peerDependencies: {
        "@wagmi/core": "2.x",
      },
      dependencies: packageJson.dependencies,
    };

    // Write the updated package.json
    await fs.writeFile(packageJsonPath, JSON.stringify(wagmiConnectorPackage, null, 2));

    // Copy the wagmi connector README
    await fs.copyFile(path.join(__dirname, "README-wagmi-connector.md"), path.join(__dirname, "README.md"));

    console.log(`Created wagmi connector package.json for version ${version}`);
  } catch (error) {
    console.error("Error creating wagmi connector package.json:", error);
    process.exit(1);
  }
}

prepareWagmiConnectorPackage();
