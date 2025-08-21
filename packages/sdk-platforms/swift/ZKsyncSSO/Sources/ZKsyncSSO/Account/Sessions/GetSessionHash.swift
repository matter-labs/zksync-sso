import Foundation
@preconcurrency import ZKsyncSSOFFI

public func getSessionHash(sessionConfigJson: String) throws -> String {
    try ZKsyncSSOFFI.getSessionHash(sessionConfigJson: sessionConfigJson)
}
