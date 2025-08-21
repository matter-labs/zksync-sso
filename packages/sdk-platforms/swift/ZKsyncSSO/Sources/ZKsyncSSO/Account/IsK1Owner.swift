import Foundation
@preconcurrency import ZKsyncSSOFFI

public struct IsK1OwnerArgs {
    let inner: ZKsyncSSOFFI.IsK1OwnerArgs
    
    public init(account: String, ownerAddress: String) {
        inner = ZKsyncSSOFFI.IsK1OwnerArgs(
            account: account,
            ownerAddress: ownerAddress
        )
    }
}

public func isK1Owner(
    args: IsK1OwnerArgs,
    config: Config
) async throws -> Bool {
    try await ZKsyncSSOFFI.isK1Owner(
        args: args.inner,
        config: config.inner
    )
}
