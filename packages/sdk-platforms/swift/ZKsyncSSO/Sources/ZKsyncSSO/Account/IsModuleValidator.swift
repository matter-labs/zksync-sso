import Foundation
@preconcurrency import ZKsyncSSOFFI

public struct IsModuleValidatorArgs {
    let inner: ZKsyncSSOFFI.IsModuleValidatorArgs
    
    public init(account: String, moduleAddress: String) {
        inner = ZKsyncSSOFFI.IsModuleValidatorArgs(
            account: account,
            moduleAddress: moduleAddress
        )
    }
}

public func isModuleValidator(
    args: IsModuleValidatorArgs,
    config: Config
) async throws -> Bool {
    try await ZKsyncSSOFFI.isModuleValidator(
        args: args.inner,
        config: config.inner
    )
}
