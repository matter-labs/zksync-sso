// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import { IPaymaster } from "account-abstraction/interfaces/IPaymaster.sol";
import { PackedUserOperation } from "account-abstraction/interfaces/PackedUserOperation.sol";
import { IEntryPoint } from "account-abstraction/interfaces/IEntryPoint.sol";

/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
/// !!!                                !!!
/// !!! THIS IS FOR DEMO PURPOSES ONLY !!!
/// !!!                                !!!
/// !!!   DO NOT COPY THIS PAYMASTER   !!!
/// !!!   FOR PRODUCTION APPLICATIONS  !!!
/// !!!                                !!!
/// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

contract TestPaymaster is IPaymaster {
  IEntryPoint public immutable entryPoint;

  constructor(address _entryPoint) {
    entryPoint = IEntryPoint(_entryPoint);
  }

  function validatePaymasterUserOp(
    PackedUserOperation calldata,
    bytes32,
    uint256
  ) external pure override returns (bytes memory context, uint256 validationData) {
    return ("", 0);
  }

  function postOp(PostOpMode, bytes calldata, uint256, uint256) external override {}

  /// @notice Deposit funds into the EntryPoint for this paymaster
  function deposit() external payable {
    entryPoint.depositTo{ value: msg.value }(address(this));
  }

  /// @notice Withdraw funds from the EntryPoint
  function withdrawTo(address payable withdrawAddress, uint256 amount) external {
    entryPoint.withdrawTo(withdrawAddress, amount);
  }

  receive() external payable {}
}
