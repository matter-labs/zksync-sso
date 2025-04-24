import Foundation
@preconcurrency import ZKsyncSSOFFI

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
    params: DeployAccountParameters,
) async throws -> Account {
    let passkeyParameters = PasskeyParameters(
        credentialRawAttestationObject: params.credentialRawAttestationObject,
        credentialRawClientDataJson: params.credentialRawClientDataJson,
        credentialId: params.credentialId,
        rpId: params.rpId
    )
    
    let uniqueAccountId = params.uniqueAccountId
    
    let account = try await ZKsyncSSOFFI.deployAccount(
        passkeyParameters: passkeyParameters,
        config: Config.default.inner
    )
    
    print("account: \(account)")
    
    return Account(
        address: account.address,
        uniqueAccountId: account.uniqueAccountId
    )
}

public func deployAccountWithUniqueId(
    params: DeployAccountParameters
) async throws -> Account {
    
    let passkeyParameters = PasskeyParameters(
        credentialRawAttestationObject: params.credentialRawAttestationObject,
        credentialRawClientDataJson: params.credentialRawClientDataJson,
        credentialId: params.credentialId,
        rpId: params.rpId
    )
    
    let uniqueAccountId = params.uniqueAccountId
    
    let account = try await ZKsyncSSOFFI.deployAccountWithUniqueId(
        passkeyParameters: passkeyParameters,
        uniqueAccountId: uniqueAccountId,
        config: Config.default.inner
    )
    
    print("account: \(account)")
    
    return Account(
        address: account.address,
        uniqueAccountId: account.uniqueAccountId
    )
}

public struct Account: Sendable {
    public var address: String
    public var uniqueAccountId: String
    
    public init(address: String, uniqueAccountId: String) {
        self.address = address
        self.uniqueAccountId = uniqueAccountId
    }
}
