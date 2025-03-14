import Foundation
import ZKsyncSSOFFI

public struct DeployAccountParameters {
    var credentialRawAttestationObject: Data
    var credentialRawClientDataJson: Data
    var credentialId: Data
    var rpId: String
    var uniqueAccountId: String
    
    public init(
        credentialRawAttestationObject: Data,
        credentialRawClientDataJson: Data,
        credentialId: Data,
        rpId: String,
        uniqueAccountId: String
    ) {
        self.credentialRawAttestationObject = credentialRawAttestationObject
        self.credentialRawClientDataJson = credentialRawClientDataJson
        self.credentialId = credentialId
        self.rpId = rpId
        self.uniqueAccountId = uniqueAccountId
    }
}

public func deployAccountWith(
    params: DeployAccountParameters
) async throws -> Account {
    
    let passkeyParameters = PasskeyParameters(
        credentialRawAttestationObject: params.credentialRawAttestationObject,
        credentialRawClientDataJson: params.credentialRawClientDataJson,
        credentialId: params.credentialId,
        rpId: params.rpId
    )
    
    let uniqueAccountId = params.uniqueAccountId
    
    let secretAccountSalt = Data([UInt8](repeating: 0, count: 32)).base64EncodedString()
    
    let account = try await ZKsyncSSOFFI.deployAccountWithUniqueId(
        passkeyParameters: passkeyParameters,
        uniqueAccountId: uniqueAccountId,
        secretAccountSalt: secretAccountSalt,
        config: Config.default.inner
    )
    
    print("account: \(account)")
    
    return Account(address: account.address, uniqueAccountId: account.uniqueAccountId)
}

public struct Account: Sendable {
    public var address: String
    public var uniqueAccountId: String
    
    public init(address: String, uniqueAccountId: String) {
        self.address = address
        self.uniqueAccountId = uniqueAccountId
    }
}
