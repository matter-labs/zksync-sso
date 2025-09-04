import Foundation
import ZKsyncSSOFFI

public struct DeployWallet {
    public var privateKeyHex: String {
        get { inner.privateKeyHex }
        set { inner.privateKeyHex = newValue }
    }

    var inner: ZKsyncSSOFFI.DeployWallet

    public init(
        privateKeyHex: String
    ) {
        inner = .init(
            privateKeyHex: privateKeyHex
        )
    }

    init(inner: ZKsyncSSOFFI.DeployWallet) {
        self.inner = inner
    }
}
