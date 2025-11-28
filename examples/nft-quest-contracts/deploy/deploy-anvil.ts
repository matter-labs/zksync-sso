import fs from "fs";
import hre from "hardhat";
import path from "path";

async function main() {
  // Use standard hardhat ethers to deploy on Anvil (localhost:8545)
  const [deployer] = await hre.ethers.getSigners();
  console.log("Deploying with:", deployer.address);

  // Simple NFT contract deployment using OpenZeppelin ERC721
  const baseTokenURI = "https://nft.zksync.dev/nft/metadata.json";
  const ZeekFactory = await hre.ethers.getContractFactory("ZeekNFTQuest");
  const nft = await ZeekFactory.deploy(baseTokenURI);
  await nft.waitForDeployment();
  const nftAddress = await nft.getAddress();
  console.log("ZeekNFTQuest deployed:", nftAddress);

  // For minimal flow we skip paymaster; rich account funds gas directly.

  // Write env for front-end consumption (duplicated pattern acceptable)
  const envFilePath = path.join(__dirname, "../../nft-quest/.env.local");
  const envContent = `NUXT_PUBLIC_CONTRACTS_NFT=${nftAddress}\n`;
  fs.writeFileSync(envFilePath, envContent, { encoding: "utf8" });
  console.log("Updated .env.local with NFT address");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
