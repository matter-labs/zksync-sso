// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import { IPaymaster, ExecutionResult, PAYMASTER_VALIDATION_SUCCESS_MAGIC } from "@matterlabs/zksync-contracts/l2/system-contracts/interfaces/IPaymaster.sol";
import { IPaymasterFlow } from "@matterlabs/zksync-contracts/l2/system-contracts/interfaces/IPaymasterFlow.sol";
import { TransactionHelper, Transaction } from "@matterlabs/zksync-contracts/l2/system-contracts/libraries/TransactionHelper.sol";

import "@matterlabs/zksync-contracts/l2/system-contracts/Constants.sol";

import "@openzeppelin/contracts/access/Ownable.sol";

/// @author Matter Labs
/// @notice This contract does not include any validations other than using the paymaster general flow.
contract RoomManagerPaymaster is IPaymaster, Ownable {
  address public immutable CONTRACT_ADDRESS;
  bytes4 public constant CREATE_ROOM_SELECTOR = bytes4(keccak256("createRoom(string)"));
  bytes4 public constant ADD_ADMIN_SELECTOR = bytes4(keccak256("addAdmin(bytes32,address)"));
  bytes4 public constant ADD_QUESTION_SELECTOR = bytes4(keccak256("addQuestion(bytes32,string)"));
  bytes4 public constant VOTE_QUESTION_SELECTOR = bytes4(keccak256("voteQuestion(bytes32,uint256,bool)"));
  bytes4 public constant TOGGLE_QUESTION_STATUS_SELECTOR = bytes4(keccak256("toggleQuestionStatus(bytes32,uint256)"));

  modifier onlyBootloader() {
    require(msg.sender == BOOTLOADER_FORMAL_ADDRESS, "Only bootloader can call this method");
    // Continue execution if called from the bootloader.
    _;
  }

  constructor(address contractAddress) {
    CONTRACT_ADDRESS = contractAddress;
  }

  function validateAndPayForPaymasterTransaction(
    bytes32,
    bytes32,
    Transaction calldata _transaction
  ) external payable onlyBootloader returns (bytes4 magic, bytes memory context) {
    // By default we consider the transaction as accepted.
    magic = PAYMASTER_VALIDATION_SUCCESS_MAGIC;
    require(_transaction.paymasterInput.length >= 4, "The standard paymaster input must be at least 4 bytes long");

    // Ensure the transaction is calling the ZeekNFTQuest contract
    require(address(uint160(_transaction.to)) == CONTRACT_ADDRESS, "Unsupported contract address");
    // Ensure the transaction is calling the `mint` function
    require(_transaction.data.length >= 4, "Transaction data is too short");
    bytes4 methodSelector = bytes4(_transaction.data[0:4]);
    require(
      methodSelector == CREATE_ROOM_SELECTOR ||
        methodSelector == ADD_ADMIN_SELECTOR ||
        methodSelector == ADD_QUESTION_SELECTOR ||
        methodSelector == VOTE_QUESTION_SELECTOR ||
        methodSelector == TOGGLE_QUESTION_STATUS_SELECTOR,
      "Unsupported method"
    );

    bytes4 paymasterInputSelector = bytes4(_transaction.paymasterInput[0:4]);
    if (paymasterInputSelector == IPaymasterFlow.general.selector) {
      // Note, that while the minimal amount of ETH needed is tx.gasPrice * tx.gasLimit,
      // neither paymaster nor account are allowed to access this context variable.
      uint256 requiredETH = _transaction.gasLimit * _transaction.maxFeePerGas;

      // The bootloader never returns any data, so it can safely be ignored here.
      (bool success, ) = payable(BOOTLOADER_FORMAL_ADDRESS).call{ value: requiredETH }("");
      require(success, "Failed to transfer tx fee to the Bootloader. Paymaster balance might not be enough.");
    } else {
      revert("Unsupported paymaster flow in paymasterParams.");
    }
  }

  function postTransaction(
    bytes calldata _context,
    Transaction calldata _transaction,
    bytes32,
    bytes32,
    ExecutionResult _txResult,
    uint256 _maxRefundedGas
  ) external payable override onlyBootloader {}

  function withdraw(address payable _to) external onlyOwner {
    uint256 balance = address(this).balance;
    (bool success, ) = _to.call{ value: balance }("");
    require(success, "Failed to withdraw funds from paymaster.");
  }

  receive() external payable {}
}
