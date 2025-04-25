import Foundation
@preconcurrency import ZKsyncSSOFFI

public func fetchAccount(
    uniqueAccountId: String,
    relyingPartyIdentifier: String
) async throws -> Account {
    let account = try await ZKsyncSSOFFI.fetchAccount(
        uniqueAccountId: uniqueAccountId,
        expectedOrigin: relyingPartyIdentifier,
        config: Config.default.inner
    )
    return Account(
        address: account.address,
        uniqueAccountId: account.uniqueAccountId
    )
}
