// SPDX-License-Identifier: Unlicensed
pragma solidity ^0.8.0;

contract RoomManager {
  struct Room {
    string name;
    address[] admins;
    mapping(address => bool) bannedUsers;
  }

  struct Question {
    address author;
    string content;
    uint256 upvotes;
    uint256 downvotes;
    bool isRead;
    mapping(address => bool) hasVoted;
  }

  mapping(bytes32 => Room) public rooms;
  mapping(bytes32 => Question[]) public roomQuestions;

  event RoomCreated(bytes32 indexed roomId, string name, address admin);
  event AdminAdded(bytes32 indexed roomId, address newAdmin);
  event UserBanned(bytes32 indexed roomId, address user);
  event UserUnbanned(bytes32 indexed roomId, address user);
  event QuestionAdded(bytes32 indexed roomId, uint256 indexed questionId, address author, string content);
  event QuestionVoted(bytes32 indexed roomId, uint256 indexed questionId, address voter, bool isUpvote);
  event QuestionStatusChanged(bytes32 indexed roomId, uint256 indexed questionId, bool isRead);

  function createRoom(string memory _name) external returns (bytes32) {
    bytes32 roomId = keccak256(abi.encodePacked(_name, msg.sender, block.timestamp));
    Room storage newRoom = rooms[roomId];
    newRoom.name = _name;
    newRoom.admins[0] = msg.sender;

    emit RoomCreated(roomId, _name, msg.sender);
    return roomId;
  }

  function isAdmin(bytes32 _roomId, address _user) public view returns (bool) {
    for (uint256 i = 0; i < rooms[_roomId].admins.length; i++) {
      if (rooms[_roomId].admins[i] == _user) {
        return true;
      }
    }
    return false;
  }

  modifier onlyAdmin(bytes32 _roomId) {
    require(isAdmin(_roomId, msg.sender), "Only admin can perform this action");
    _;
  }

  modifier notBanned(bytes32 _roomId) {
    require(!rooms[_roomId].bannedUsers[msg.sender], "User is banned from this room");
    _;
  }

  function addAdmin(bytes32 _roomId, address _newAdmin) external onlyAdmin(_roomId) {
    require(_newAdmin != address(0), "Invalid admin address");
    require(!isAdmin(_roomId, _newAdmin), "Address is already an admin");

    rooms[_roomId].admins.push(_newAdmin);
    emit AdminAdded(_roomId, _newAdmin);
  }

  function banUser(bytes32 _roomId, address _user) external onlyAdmin(_roomId) {
    require(!isAdmin(_roomId, _user), "Cannot ban admin");
    rooms[_roomId].bannedUsers[_user] = true;
    emit UserBanned(_roomId, _user);
  }

  function unbanUser(bytes32 _roomId, address _user) external onlyAdmin(_roomId) {
    rooms[_roomId].bannedUsers[_user] = false;
    emit UserUnbanned(_roomId, _user);
  }

  function isBanned(bytes32 _roomId, address _user) external view returns (bool) {
    return rooms[_roomId].bannedUsers[_user];
  }

  // Question Management Functions

  function addQuestion(bytes32 _roomId, string memory _content) external notBanned(_roomId) {
    uint256 questionId = roomQuestions[_roomId].length;
    Question storage newQuestion = roomQuestions[_roomId].push();
    newQuestion.author = msg.sender;
    newQuestion.content = _content;
    newQuestion.upvotes = 0;
    newQuestion.downvotes = 0;
    newQuestion.isRead = false;

    emit QuestionAdded(_roomId, questionId, msg.sender, _content);
  }

  function voteQuestion(bytes32 _roomId, uint256 _questionId, bool _isUpvote) external notBanned(_roomId) {
    require(_questionId < roomQuestions[_roomId].length, "Invalid question ID");
    Question storage question = roomQuestions[_roomId][_questionId];
    require(!question.hasVoted[msg.sender], "User has already voted");

    if (_isUpvote) {
      question.upvotes++;
    } else {
      question.downvotes++;
    }

    question.hasVoted[msg.sender] = true;
    emit QuestionVoted(_roomId, _questionId, msg.sender, _isUpvote);
  }

  function toggleQuestionStatus(bytes32 _roomId, uint256 _questionId) external onlyAdmin(_roomId) {
    require(_questionId < roomQuestions[_roomId].length, "Invalid question ID");
    Question storage question = roomQuestions[_roomId][_questionId];
    question.isRead = !question.isRead;
    emit QuestionStatusChanged(_roomId, _questionId, question.isRead);
  }

  function getQuestionCount(bytes32 _roomId) external view returns (uint256) {
    return roomQuestions[_roomId].length;
  }
}
