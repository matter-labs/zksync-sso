import Foundation
@preconcurrency import ZKsyncSSOFFI

public struct DeployAccountParameters {
    var credentialRawAttestationObject: Data
    var credentialRawClientDataJson: Data
    var credentialId: Data
    var rpId: String
    var uniqueAccountId: String
    var initialK1Owners: [String]?
    var initialSessionConfigJson: String?

    public init(
        credentialRawAttestationObject: Data,
        credentialRawClientDataJson: Data,
        credentialId: Data,
        rpId: String,
        uniqueAccountId: String,
        initialK1Owners: [String]?,
        initialSessionConfigJson: String?
    ) {
        self.credentialRawAttestationObject = credentialRawAttestationObject
        self.credentialRawClientDataJson = credentialRawClientDataJson
        self.credentialId = credentialId
        self.rpId = rpId
        self.uniqueAccountId = uniqueAccountId
        self.initialK1Owners = initialK1Owners
        self.initialSessionConfigJson = initialSessionConfigJson
    }
}

public func deployAccountWith(
    params: DeployAccountParameters,
) async throws -> Account {
    let passkeyParameters = PasskeyParameters(
        credentialRawAttestationObject: params.credentialRawAttestationObject,
        credentialRawClientDataJson: params.credentialRawClientDataJson,
        credentialId: params.credentialId,
        rpId: .apple(params.rpId),
    )

    let account = try await ZKsyncSSOFFI.deployAccountWithUniqueId(
        passkeyParameters: passkeyParameters,
        uniqueAccountId: params.uniqueAccountId,
        initialK1Owners: params.initialK1Owners,
        initialSessionConfigJson: params.initialSessionConfigJson,
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
        rpId: .apple(params.rpId),
    )

    let uniqueAccountId = params.uniqueAccountId

    let account = try await ZKsyncSSOFFI.deployAccountWithUniqueId(
        passkeyParameters: passkeyParameters,
        uniqueAccountId: uniqueAccountId,
        initialK1Owners: nil,
        initialSessionConfigJson: nil,
        config: Config.default.inner
    )

    return Account(
        address: account.address,
        uniqueAccountId: account.uniqueAccountId
    )
}
