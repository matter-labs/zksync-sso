import Foundation
@preconcurrency import ZKsyncSSOFFI

public func fundAccount(
    address: String,
    amount: String,
    config: Config
) async throws {
    try await ZKsyncSSOFFI.fundAccount(
        address: address,
        amount: amount,
        config: config.inner
    )
}
