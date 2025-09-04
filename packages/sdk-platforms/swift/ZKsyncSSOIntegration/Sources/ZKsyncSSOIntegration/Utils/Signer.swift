import Foundation
import ZKsyncSSO

public protocol Signer: Equatable, Hashable, Identifiable {
    var address: String { get }
    var privateKeyHex: String { get }

    init(address: String, privateKeyHex: String)
}

extension Signer {
    public var id: String {
        address
    }
}

extension Signer {
    static func deriveFrom(privateKeyHex: String) -> Result<Self, any Error> {
        Result {
            try ZKsyncSSO.deriveAddressFrom(privateKeyHex: privateKeyHex)
        }
        .map { address in
            Self(address: address, privateKeyHex: privateKeyHex)
        }
    }
}