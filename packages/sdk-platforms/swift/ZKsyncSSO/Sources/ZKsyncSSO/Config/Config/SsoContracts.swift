import Foundation
import ZKsyncSSOFFI

public struct SsoContracts {
    public var accountFactory: String { inner.accountFactory }
    public var passkey: String { inner.passkey }
    public var session: String { inner.session }
    public var accountPaymaster: String { inner.accountPaymaster }
    public var recovery: String { inner.recovery }

    var inner: ZKsyncSSOFFI.SsoContracts

    public init(
        accountFactory: String,
        passkey: String,
        session: String,
        accountPaymaster: String,
        recovery: String
    ) {
        inner = .init(
            accountFactory: accountFactory,
            passkey: passkey,
            session: session,
            accountPaymaster: accountPaymaster,
            recovery: recovery
        )
    }

    init(inner: ZKsyncSSOFFI.SsoContracts) {
        self.inner = inner
    }
}
