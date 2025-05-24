import Foundation
@preconcurrency import ZKsyncSSOFFI

public func initLogger(bundleIdentifier: String) {
    ZKsyncSSOFFI.initAppleLogger(bundleIdentifier: bundleIdentifier)
}
