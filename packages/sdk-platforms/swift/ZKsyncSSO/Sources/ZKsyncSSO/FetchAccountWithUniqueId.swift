import Foundation
@preconcurrency import ZKsyncSSOFFI

public func fetchAccountWith(uniqueAccountId: String) async throws -> Account {
    let account = try await ZKsyncSSOFFI.getAccountByUserId(
        uniqueAccountId: uniqueAccountId,
        config: Config.default.inner
    )
    print("account: \(account)")
    return Account(
        address: account.address,
        uniqueAccountId: account.uniqueAccountId
    )
}
