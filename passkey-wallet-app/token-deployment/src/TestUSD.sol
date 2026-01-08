// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract TestUSD is ERC20 {
  constructor() ERC20("USD Test Token", "USD") {
    _mint(msg.sender, 1000000 * 10 ** decimals());
  }
}
