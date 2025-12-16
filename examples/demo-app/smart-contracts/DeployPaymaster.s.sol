// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import { Script } from "forge-std/Script.sol";
import { console } from "forge-std/console.sol";
import { TestPaymaster } from "./TestPaymaster.sol";

contract DeployPaymaster is Script {
  function run() public {
    vm.startBroadcast();

    TestPaymaster paymaster = new TestPaymaster();

    vm.stopBroadcast();

    console.log("TestPaymaster:", address(paymaster));
  }
}
