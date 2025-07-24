import Foundation
import ZKsyncSSOFFI

public struct SsoContracts {
    let inner: ZKsyncSSOFFI.SsoContracts
    
    public init(
        accountFactory: String,
        passkey: String,
        session: String,
        accountPaymaster: String,
        recovery: String
    ) {
        self.inner = .init(
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
