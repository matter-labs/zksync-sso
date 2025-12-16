// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import { IPaymaster } from "account-abstraction/interfaces/IPaymaster.sol";
import { PackedUserOperation } from "account-abstraction/interfaces/PackedUserOperation.sol";

/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
/// !!!                                !!!
/// !!! THIS IS FOR DEMO PURPOSES ONLY !!!
/// !!!                                !!!
/// !!!   DO NOT COPY THIS PAYMASTER   !!!
/// !!!   FOR PRODUCTION APPLICATIONS  !!!
/// !!!                                !!!
/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

contract TestPaymaster is IPaymaster {
  function validatePaymasterUserOp(
    PackedUserOperation calldata,
    bytes32,
    uint256
  ) external pure override returns (bytes memory context, uint256 validationData) {
    return ("", 0);
  }

  function postOp(PostOpMode, bytes calldata, uint256, uint256) external override {}

  receive() external payable {}
}
