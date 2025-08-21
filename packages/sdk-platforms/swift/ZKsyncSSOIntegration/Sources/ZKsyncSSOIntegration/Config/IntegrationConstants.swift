import Foundation
import ZKsyncSSO

// MARK: - Integration Constants

public struct IntegrationConstants {
    // Test addresses
    public static let transferSessionTarget = "0xdeBbD4CE2Bd6BD869D3ac93666A0D5F4fc06FC72"

    // Signers (private key + address pairs)
    public static let accountOwner =
        RichWallet(
            address: "0x6a34Ea49c29BF7Cce95F51E7F0f419831Ad5dBC6",
            privateKeyHex: "0x5de4111afa1a4b94908f83103c3a57e0c3c9e9da2dd5a02a84e9fde30d7e96c3"
        )
    public static let sessionOwner = RichWallet.six

    // Salts
    public static let randomSaltStr = "sdk-test-factory-replication-010"

    // Test session configuration
    public static let expiresAt: UInt64 = 1_749_040_108
    public static let feeLimitLifetime = "100000000000000000"  // 0.1 ETH
    public static let maxValuePerUse = "10000000000000000"  // 0.01 ETH

    public static var secondSessionOwnerAddress: String { accountOwner.address }
    public static var secondSessionOwnerPrivateKey: String { accountOwner.privateKeyHex }
    public static let secondSessionOwner = {
        return accountOwner
    }()

    public static let secondSessionFeeLimit = "50000000000000000"  // 0.05 ETH
    public static let secondSessionMaxValuePerUse = "5000000000000000"  // 0.005 ETH
    public static let vitalikAddress = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
    public static let expectedSecondSessionHash =
    "0xe890af4a9040b7eb7f58f1cc399863500e8291b6dfd088fa5d7ec2ef8b35306b"

    public static let sessionSendTestDeploySigner = RichWallet.four

    public static let sessionSendTestOwner = RichWallet.three
    public static var sessionSendTestOwnerAddress: String {
        sessionSendTestOwner.address
    }

    public static let sessionSendTestTransferAmount = "5000000000000000"  // 0.005 ETH
    public static let sessionSendTestTransferTarget = "0xdeBbD4CE2Bd6BD869D3ac93666A0D5F4fc06FC72"
    public static let sessionSendTestUniqueAccountId = "session-send-test-002"

    public static let sessionSendTestSessionOwner = RichWallet.five
    public static var sessionSendTestSessionOwnerPrivateKey: String {
        sessionSendTestSessionOwner.privateKeyHex
    }
    public static var sessionSendTestSessionOwnerAddress: String {
        sessionSendTestSessionOwner.address
    }

    public static let sessionSendTestExpiresAt: UInt64 = 1_767_225_600  // January 1, 2026, 00:00:00 UTC
    public static let sessionSendTestExpectedSessionKeyBytes: [UInt8] = [
        139, 58, 53, 12, 245, 195, 76, 145, 148, 202, 133, 130, 154, 45,
        240, 236, 49, 83, 190, 3, 24, 181, 226, 211, 52, 142, 135, 32, 146,
        237, 255, 186,
    ]

    // Deterministic deployed account address (from consistent deployment with same salt/config)
    public static let deployedAccountAddress = "0x177B4fe98b5F6ee253EFfFe1226c9C3E9f5e37cb"

    public static let sessionSendTestDeployedAccountAddress =
        "0x58bFc7e1B92Fb660999DFA152505288AA1f5D9A6"

    public static func createSessionConfigJson(
        sessionOwner: any Signer,
        expiresAt: String,
        feeLimitLifetime: String,
        target: String,
        maxValuePerUse: String,
    ) -> String {
        return """
            {
                "signer": "\(sessionOwner.address)",
                "expiresAt": "\(expiresAt)",
                "feeLimit": {
                    "limitType": "Lifetime",
                    "limit": "\(feeLimitLifetime)",
                    "period": "0"
                },
                "callPolicies": [],
                "transferPolicies": [{
                    "target": "\(target)",
                    "maxValuePerUse": "\(maxValuePerUse)",
                    "valueLimit": {
                        "limitType": "Unlimited", 
                        "limit": "0",
                        "period": "0"
                    }
                }]
            }
            """
    }
}
