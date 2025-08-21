import Foundation
import ZKsyncSSOFFI

public func deriveAddressFrom(privateKeyHex: String) throws -> String {
    try privateKeyToAddress(privateKey: privateKeyHex)
}
