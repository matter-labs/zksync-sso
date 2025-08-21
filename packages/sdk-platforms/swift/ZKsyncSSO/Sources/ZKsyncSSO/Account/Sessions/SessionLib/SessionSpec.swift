import Foundation
import ZKsyncSSOFFI

public struct SessionSpec: Codable, Equatable, Hashable, Sendable {
    public var signer: String
    public var expiresAt: String
    public var feeLimit: UsageLimit
    public var callPolicies: [CallSpec]
    public var transferPolicies: [TransferSpec]

    public init(
        signer: String,
        expiresAt: String,
        feeLimit: UsageLimit,
        callPolicies: [CallSpec],
        transferPolicies: [TransferSpec]
    ) {
        self.signer = signer
        self.expiresAt = expiresAt
        self.feeLimit = feeLimit
        self.callPolicies = callPolicies
        self.transferPolicies = transferPolicies
    }

    public func toJsonString(pretty: Bool = false) throws -> String {
        let encoder = JSONEncoder()
        if pretty { encoder.outputFormatting = [.prettyPrinted, .sortedKeys] }
        let data = try encoder.encode(self)
        return String(decoding: data, as: UTF8.self)
    }
    
    public static func fromJsonString(_ jsonString: String) throws -> SessionSpec {
        let data = jsonString.data(using: .utf8)!
        return try JSONDecoder().decode(SessionSpec.self, from: data)
    }
    
    public func sessionHash() throws -> String {
        let sessionConfigJson = try toJsonString()
        return try getSessionHash(sessionConfigJson: sessionConfigJson)
    }
}
