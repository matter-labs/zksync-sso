// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC721/IERC721.sol";
import "@openzeppelin/contracts/utils/cryptography/EIP712.sol";
import "@openzeppelin/contracts/utils/cryptography/SignatureChecker.sol";
import "@openzeppelin/contracts/interfaces/IERC1271.sol"; // For EIP-1271

/**
 * @title SimpleMarketplace
 * @dev A basic marketplace for ERC721 NFTs supporting EIP-1271 for listings.
 */
contract SimpleMarketplace is EIP712 {
  // --- Structs ---
  struct Listing {
    address seller;
    address nftContract;
    uint256 tokenId;
    uint256 price; // in wei
    bool active;
  }

  // --- State Variables ---
  mapping(address => mapping(uint256 => Listing)) public listings; // nftContract -> tokenId -> Listing
  mapping(address => uint256) public sellerNonces; // To prevent signature replay for listings

  // --- EIP-712 Domain ---
  // solhint-disable-next-line var-name-mixedcase
  bytes32 private immutable _LIST_REQUEST_TYPEHASH =
    keccak256("ListRequest(address seller,address nftContract,uint256 tokenId,uint256 price,uint256 nonce)");

  // --- Events ---
  event ItemListed(address indexed seller, address indexed nftContract, uint256 indexed tokenId, uint256 price);
  event ItemSold(
    address indexed seller,
    address indexed buyer,
    address indexed nftContract,
    uint256 tokenId,
    uint256 price
  );
  event ItemUnlisted(address indexed seller, address indexed nftContract, uint256 tokenId);
  event PriceUpdated(address indexed seller, address indexed nftContract, uint256 tokenId, uint256 newPrice);

  // --- Constructor ---
  constructor(string memory name, string memory version) EIP712(name, version) {}

  // --- Functions ---

  /**
   * @notice Lists an NFT for sale directly by the owner.
   * @dev The NFT must be approved for this marketplace contract.
   * @param _nftContract The address of the NFT contract.
   * @param _tokenId The ID of the token to list.
   * @param _price The price in wei.
   */
  function listItem(address _nftContract, uint256 _tokenId, uint256 _price) public {
    IERC721 nft = IERC721(_nftContract);
    require(nft.ownerOf(_tokenId) == msg.sender, "Not owner");
    require(
      nft.getApproved(_tokenId) == address(this) || nft.isApprovedForAll(msg.sender, address(this)),
      "Marketplace not approved"
    );
    require(_price > 0, "Price must be > 0");

    listings[_nftContract][_tokenId] = Listing({
      seller: msg.sender,
      nftContract: _nftContract,
      tokenId: _tokenId,
      price: _price,
      active: true
    });

    emit ItemListed(msg.sender, _nftContract, _tokenId, _price);
  }

  /**
   * @notice Lists an NFT using a signature, typically for smart contract sellers.
   * @dev This allows gasless listing submission. The NFT must still be approved
   * for the marketplace contract for a sale to execute.
   * @param _seller The address of the seller (can be a smart account).
   * @param _nftContract The address of the NFT contract.
   * @param _tokenId The ID of the token to list.
   * @param _price The price in wei.
   * @param _signature The EIP-712 signature from the seller (or its owner).
   */
  function listItemWithSignature(
    address _seller,
    address _nftContract,
    uint256 _tokenId,
    uint256 _price,
    bytes calldata _signature
  ) public {
    require(_seller != address(0), "Invalid seller");
    require(_nftContract != address(0), "Invalid NFT contract");
    require(_price > 0, "Price must be > 0");

    uint256 currentNonce = sellerNonces[_seller];
    bytes32 structHash = _hashListRequest(_seller, _nftContract, _tokenId, _price, currentNonce);

    // EIP-1271 check if seller is a contract
    // For EOAs, SignatureChecker.isValidSignatureNow will recover and check.
    // For contracts, it will call `isValidSignature` on the contract.
    require(SignatureChecker.isValidSignatureNow(_seller, structHash, _signature), "Invalid signature");

    // The actual owner check and approval check are deferred until `buyItem`.
    // However, it's good practice for the seller to ensure they own it
    // and approve the marketplace *before or around the time of listing*.

    listings[_nftContract][_tokenId] = Listing({
      seller: _seller,
      nftContract: _nftContract,
      tokenId: _tokenId,
      price: _price,
      active: true
    });

    sellerNonces[_seller]++; // Increment nonce after successful use

    emit ItemListed(_seller, _nftContract, _tokenId, _price);
  }

  /**
   * @notice Buys a listed NFT.
   * @param _nftContract The address of the NFT contract.
   * @param _tokenId The ID of the token to buy.
   */
  function buyItem(address _nftContract, uint256 _tokenId) public payable {
    Listing storage currentListing = listings[_nftContract][_tokenId];
    require(currentListing.active, "Item not listed or already sold");
    require(msg.value == currentListing.price, "Incorrect price");

    IERC721 nft = IERC721(_nftContract);
    // Ensure the marketplace is still approved and seller is owner
    require(
      nft.getApproved(_tokenId) == address(this) || nft.isApprovedForAll(currentListing.seller, address(this)),
      "Marketplace not approved for transfer"
    );
    require(nft.ownerOf(_tokenId) == currentListing.seller, "Seller no longer owns NFT");

    address seller = currentListing.seller;
    uint256 price = currentListing.price;

    currentListing.active = false;

    // Transfer NFT
    nft.safeTransferFrom(seller, msg.sender, _tokenId);

    // Transfer funds
    (bool success, ) = seller.call{ value: price }("");
    require(success, "Payment failed");

    emit ItemSold(seller, msg.sender, _nftContract, _tokenId, price);
  }

  /**
   * @notice Unlists an item.
   * @param _nftContract The address of the NFT contract.
   * @param _tokenId The ID of the token to unlist.
   */
  function unlistItem(address _nftContract, uint256 _tokenId) public {
    Listing storage currentListing = listings[_nftContract][_tokenId];
    require(currentListing.seller == msg.sender, "Not seller");
    require(currentListing.active, "Not listed");

    currentListing.active = false;
    emit ItemUnlisted(msg.sender, _nftContract, _tokenId);
  }

  /**
   * @notice Updates the price of a listed item.
   * @param _nftContract The address of the NFT contract.
   * @param _tokenId The ID of the token.
   * @param _newPrice The new price in wei.
   */
  function updatePrice(address _nftContract, uint256 _tokenId, uint256 _newPrice) public {
    Listing storage currentListing = listings[_nftContract][_tokenId];
    require(currentListing.seller == msg.sender, "Not seller");
    require(currentListing.active, "Not listed");
    require(_newPrice > 0, "Price must be > 0");

    currentListing.price = _newPrice;
    emit PriceUpdated(msg.sender, _nftContract, _tokenId, _newPrice);
  }

  /**
   * @notice Hashes the ListRequest struct for EIP-712 signing.
   */
  function _hashListRequest(
    address _seller,
    address _nftContract,
    uint256 _tokenId,
    uint256 _price,
    uint256 _nonce
  ) internal view returns (bytes32) {
    return
      _hashTypedDataV4(keccak256(abi.encode(_LIST_REQUEST_TYPEHASH, _seller, _nftContract, _tokenId, _price, _nonce)));
  }

  /**
   * @notice Returns the current EIP-712 domain separator.
   */
  function getDomainSeparator() external view returns (bytes32) {
    return _domainSeparatorV4();
  }

  /**
   * @notice Returns the type hash for the ListRequest struct.
   */
  function getListRequestTypeHash() external view returns (bytes32) {
    return _LIST_REQUEST_TYPEHASH;
  }
}
