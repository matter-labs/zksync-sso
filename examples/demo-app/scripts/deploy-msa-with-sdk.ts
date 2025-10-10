import { promises as fs } from "fs";
import * as path from "path";
import { fileURLToPath } from "url";
import { Provider, Wallet, ContractFactory } from "zksync-ethers";

// Get __dirname equivalent in ES modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Configuration
const RPC_URL = process.env.RPC_URL || "http://localhost:8011";
const PRIVATE_KEY = process.env.PRIVATE_KEY || "0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110";

// Contract artifacts path (absolute)
const ARTIFACTS_PATH = "/home/colinbellmore/Documents/zksync-sso/packages/erc4337-contracts/out";

async function loadArtifact(contractName: string, fileName?: string) {
  const file = fileName || contractName;
  const artifactPath = path.join(ARTIFACTS_PATH, `${file}.sol`, `${contractName}.json`);
  const artifactJson = await fs.readFile(artifactPath, "utf8");
  const artifact = JSON.parse(artifactJson);

  return {
    abi: artifact.abi,
    bytecode: artifact.bytecode.object,
  };
}

async function deployContract(
  wallet: Wallet,
  contractName: string,
  constructorArgs: any[] = [],
  fileName?: string,
) {
  console.log(`\n📦 Deploying ${contractName}...`);

  const artifact = await loadArtifact(contractName, fileName);
  const factory = new ContractFactory(artifact.abi, artifact.bytecode, wallet);

  const contract = await factory.deploy(...constructorArgs);
  await contract.waitForDeployment();

  const address = await contract.getAddress();
  console.log(`✅ ${contractName}: ${address}`);

  return contract;
}

async function deployWithProxy(
  wallet: Wallet,
  implName: string,
  implArgs: any[] = [],
  proxyAdmin?: string,
  fileName?: string,
) {
  // Deploy implementation
  const implementation = await deployContract(wallet, implName, implArgs, fileName);
  const implAddress = await implementation.getAddress();

  // Deploy proxy
  const proxyAdmin_ = proxyAdmin || wallet.address;
  const proxy = await deployContract(
    wallet,
    "TransparentUpgradeableProxy",
    [implAddress, proxyAdmin_, "0x"],
    "TransparentUpgradeableProxy",
  );

  return { implementation, proxy };
}

async function main() {
  console.log("🚀 Deploying MSA Factory and modules with zkSync SDK...\n");

  // Initialize provider and wallet
  const provider = new Provider(RPC_URL);
  const wallet = new Wallet(PRIVATE_KEY, provider);

  console.log(`📍 Deployer: ${wallet.address}`);
  console.log(`🌐 RPC URL: ${RPC_URL}`);
  console.log(`⛓️  Chain ID: ${(await provider.getNetwork()).chainId}`);

  const balance = await wallet.getBalance();
  console.log(`💰 Balance: ${balance.toString()} wei`);

  // Deploy validators with proxies
  const { proxy: eoaValidator } = await deployWithProxy(wallet, "EOAKeyValidator");
  const { proxy: sessionValidator } = await deployWithProxy(wallet, "SessionKeyValidator");
  const { proxy: webauthnValidator } = await deployWithProxy(wallet, "WebAuthnValidator");

  // Deploy GuardianExecutor
  const { proxy: guardianExecutor } = await deployWithProxy(wallet, "GuardianExecutor");

  // Deploy ModularSmartAccount implementation
  const accountImplementation = await deployContract(wallet, "ModularSmartAccount");

  // Deploy UpgradeableBeacon
  const beacon = await deployContract(
    wallet,
    "UpgradeableBeacon",
    [await accountImplementation.getAddress(), wallet.address],
  );

  // Deploy MSAFactory with proxy
  const factoryImpl = await deployContract(
    wallet,
    "MSAFactory",
    [await beacon.getAddress()],
  );

  const factoryProxy = await deployContract(
    wallet,
    "TransparentUpgradeableProxy",
    [await factoryImpl.getAddress(), wallet.address, "0x"],
    "TransparentUpgradeableProxy",
  );

  // Collect all addresses
  const addresses = {
    rpcUrl: RPC_URL,
    chainId: Number((await provider.getNetwork()).chainId),
    deployer: wallet.address,
    eoaValidator: await eoaValidator.getAddress(),
    sessionValidator: await sessionValidator.getAddress(),
    webauthnValidator: await webauthnValidator.getAddress(),
    guardianExecutor: await guardianExecutor.getAddress(),
    accountImplementation: await accountImplementation.getAddress(),
    beacon: await beacon.getAddress(),
    factory: await factoryProxy.getAddress(),
  };

  // Save to contracts.json
  const outputPath = path.join(__dirname, "../contracts.json");
  await fs.writeFile(outputPath, JSON.stringify(addresses, null, 2));
  console.log(`\n💾 Saved addresses to ${outputPath}`);

  // Copy to public directory
  const publicPath = path.join(__dirname, "../public/contracts.json");
  await fs.writeFile(publicPath, JSON.stringify(addresses, null, 2));
  console.log(`💾 Copied to ${publicPath}`);

  console.log("\n🎉 Deployment complete!");
  console.log("\n📝 Contract Addresses:");
  console.log(JSON.stringify(addresses, null, 2));
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("❌ Deployment failed:", error);
    process.exit(1);
  });
