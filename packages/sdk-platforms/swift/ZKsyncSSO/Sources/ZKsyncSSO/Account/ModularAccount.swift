import Foundation
@preconcurrency import ZKsyncSSOFFI

public struct CredentialDetails: Sendable {
    public var id: String
    public var publicKey: Data

    public init(id: String, publicKey: Data) {
        self.id = id
        self.publicKey = publicKey
    }
}

public struct PasskeyModuleArgs: Sendable {
    public var location: String
    public var credential: CredentialDetails
    public var expectedOrigin: String?

    public init(location: String, credential: CredentialDetails, expectedOrigin: String? = nil) {
        self.location = location
        self.credential = credential
        self.expectedOrigin = expectedOrigin
    }
}

public struct PaymasterParams: Sendable {
    public var paymasterAddress: String
    public var paymasterInput: String?

    public init(paymasterAddress: String, paymasterInput: String? = nil) {
        self.paymasterAddress = paymasterAddress
        self.paymasterInput = paymasterInput
    }
}

// Re-export SessionModuleArgs from ZKsyncSSOFFI to avoid having duplicate types
public typealias SessionModuleArgs = ZKsyncSSOFFI.SessionModuleArgs

public struct DeployModularAccountParameters {
    public var installNoDataModules: [String]
    public var owners: [String]
    public var sessionModule: SessionModuleArgs?
    public var paymaster: PaymasterParams?
    public var passkeyModule: PasskeyModuleArgs?
    public var uniqueAccountId: String?

    public init(
        installNoDataModules: [String] = [],
        owners: [String],
        sessionModule: SessionModuleArgs? = nil,
        paymaster: PaymasterParams? = nil,
        passkeyModule: PasskeyModuleArgs? = nil,
        uniqueAccountId: String? = nil
    ) {
        self.installNoDataModules = installNoDataModules
        self.owners = owners
        self.sessionModule = sessionModule
        self.paymaster = paymaster
        self.passkeyModule = passkeyModule
        self.uniqueAccountId = uniqueAccountId
    }
}

public struct DeployedModularAccountDetails: Sendable {
    public var address: String
    public var uniqueAccountId: String
    public var transactionReceiptJson: String

    public init(address: String, uniqueAccountId: String, transactionReceiptJson: String) {
        self.address = address
        self.uniqueAccountId = uniqueAccountId
        self.transactionReceiptJson = transactionReceiptJson
    }
}

public func deployModularAccount(
    params: DeployModularAccountParameters,
    config: Config
) async throws -> DeployedModularAccountDetails {
    let ffiPaymaster = params.paymaster.map { paymaster in
        ZKsyncSSOFFI.PaymasterParams(
            paymasterAddress: paymaster.paymasterAddress,
            paymasterInput: paymaster.paymasterInput
        )
    }

    let ffiPasskeyModule = params.passkeyModule.map { passkeyModule in
        ZKsyncSSOFFI.PasskeyModuleArgs(
            location: passkeyModule.location,
            credential: ZKsyncSSOFFI.CredentialDetails(
                id: passkeyModule.credential.id,
                publicKey: passkeyModule.credential.publicKey
            ),
            expectedOrigin: passkeyModule.expectedOrigin
        )
    }

    let args = ZKsyncSSOFFI.DeployModularAccountArgs(
        installNoDataModules: params.installNoDataModules,
        owners: params.owners,
        sessionModule: params.sessionModule,
        paymaster: ffiPaymaster,
        passkeyModule: ffiPasskeyModule,
        uniqueAccountId: params.uniqueAccountId
    )

    let result = try await ZKsyncSSOFFI.deployModularAccount(
        args: args,
        config: config.inner
    )

    return DeployedModularAccountDetails(
        address: result.address,
        uniqueAccountId: result.uniqueAccountId,
        transactionReceiptJson: result.transactionReceiptJson
    )
}
