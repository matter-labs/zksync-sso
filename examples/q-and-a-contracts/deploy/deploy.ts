import { formatEther, parseEther } from "ethers";
import fs from "fs";
import { HardhatRuntimeEnvironment } from "hardhat/types";
import path from "path";

import { deployContract, getProvider, getWallet } from "./utils";

export default async function (hre: HardhatRuntimeEnvironment) {
  const provider = getProvider();

  const roomManager = await deployContract("RoomManager", []);

  const paymasterContract = await deployContract("RoomManagerPaymaster", [await roomManager.getAddress()]);

  console.log("ROOM MANAGER CONTRACT: ", await roomManager.getAddress());
  console.log("PAYMASTER CONTRACT: ", await paymasterContract.getAddress());

  if (hre.network.config.ethNetwork.includes("localhost")) {
    // Update the .env.local file with the contract addresses for NFT Quest app
    const envFilePath = path.join(__dirname, "../../q-and-a/.env.local");

    // Check if the .env.local file exists, if not, create it
    if (!fs.existsSync(envFilePath)) {
      fs.writeFileSync(envFilePath, "", { encoding: "utf8" });
      console.log(`.env.local file has been created at ${envFilePath}`);
    }
    const roomManagerAddress = await roomManager.getAddress();
    const paymasterContractAddress = await paymasterContract.getAddress();

    const envContent = `NUXT_PUBLIC_CONTRACTS_ROOM=${roomManagerAddress}\nNUXT_PUBLIC_CONTRACTS_PAYMASTER=${paymasterContractAddress}\n`;

    fs.writeFileSync(envFilePath, envContent, { encoding: "utf8" });
    console.log(`.env.local file has been updated at ${envFilePath}`);
  }

  // fund the paymaster contract with enough ETH to pay for transactions
  const wallet = getWallet();
  await (
    await wallet.sendTransaction({
      to: paymasterContract.target,
      value: parseEther("1"),
    })
  ).wait();

  const paymasterBalance = await provider.getBalance(paymasterContract.target.toString());
  console.log(
    `\nPaymaster ETH balance is now ${formatEther(
      paymasterBalance.toString(),
    )}`,
  );
}
