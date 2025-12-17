// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import { Script } from "forge-std/Script.sol";
import { console } from "forge-std/console.sol";
import { TestPaymaster } from "./TestPaymaster.sol";

contract DeployPaymaster is Script {
  // EntryPoint v0.8 canonical address
  address constant ENTRY_POINT = 0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108;

  function run() public {
    vm.startBroadcast();

    TestPaymaster paymaster = new TestPaymaster(ENTRY_POINT);

    vm.stopBroadcast();

    console.log("TestPaymaster:", address(paymaster));
    console.log("EntryPoint:", ENTRY_POINT);
  }
}
