import { parseEther } from "ethers";
import { Contract, Wallet } from "zksync-ethers";

import { deployContract, getWallet, LOCAL_RICH_WALLETS } from "../deploy/utils";

describe("RoomManager", function () {
  let roomManagerContract: Contract;
  let paymaster: Contract;
  let ownerWallet: Wallet;

  before(async function () {
    ownerWallet = getWallet(LOCAL_RICH_WALLETS[0].privateKey);

    roomManagerContract = await deployContract(
      "RoomManager",
      [],
      { wallet: ownerWallet, silent: true },
    );

    paymaster = await deployContract(
      "RoomManagerPaymaster",
      [await roomManagerContract.getAddress()],
      { wallet: ownerWallet, silent: true });

    const tx = await ownerWallet.sendTransaction({
      to: await paymaster.getAddress(),
      value: parseEther("0.042"),
    });
    await tx.wait();
  });

  it("create a new room", async function () {
    const tx = await roomManagerContract.createRoom("test room name");
    await tx.wait();
  });
});
