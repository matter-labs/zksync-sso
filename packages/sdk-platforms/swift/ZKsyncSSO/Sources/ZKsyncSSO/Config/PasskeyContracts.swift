import Foundation
import ZKsyncSSOFFI

public struct PasskeyContracts {
    let inner: ZKsyncSSOFFI.PasskeyContracts
    
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
    
    init(inner: ZKsyncSSOFFI.PasskeyContracts) {
        self.inner = inner
    }
}
