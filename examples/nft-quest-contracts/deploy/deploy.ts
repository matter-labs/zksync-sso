import { formatEther, parseEther } from "ethers";

import { deployContract, getProvider, getWallet } from "./utils";

export default async function () {
  const provider = getProvider();

  const baseTokenURI = "https://nft.zksync.dev/nft/metadata.json";
  const nftContract = await deployContract("ZeekNFTQuest", [baseTokenURI]);

  const paymasterContract = await deployContract("NFTQuestPaymaster", [await nftContract.getAddress()]);

  console.log("NFT CONTRACT: ", await nftContract.getAddress());
  console.log("PAYMASTER CONTRACT: ", await paymasterContract.getAddress());

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
