import Foundation
@preconcurrency import ZKsyncSSOFFI

public func fetchAccountWith(uniqueAccountId: String, secretAccountSalt: Data) async throws -> Account {
  let secretAccountSalt = secretAccountSalt.base64EncodedString()
  let account = try await ZKsyncSSOFFI.getAccountByUserId(
    uniqueAccountId: uniqueAccountId,
    secretAccountSalt: secretAccountSalt,
    config: Config.default.inner
  )
  print("account: \(account)")
  return Account(address: account.address, uniqueAccountId: account.uniqueAccountId)
}
