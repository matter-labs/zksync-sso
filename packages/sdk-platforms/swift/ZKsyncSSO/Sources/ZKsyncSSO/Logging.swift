import Foundation
@preconcurrency import ZKsyncSSOFFI

/// An enum representing the available verbosity level filters of the logger.
public enum LogLevel {
  
    /// Corresponds to the `Error` log level.
    case error
  
    /// Corresponds to the `Warn` log level.
    case warn
  
    /// Corresponds to the `Info` log level.
    case info
  
    /// Corresponds to the `Debug` log level.
    case debug
  
    /// Corresponds to the `Trace` log level.
    case trace
}

extension LogLevel {
    fileprivate var ffi: ZKsyncSSOFFI.LogLevel {
        switch self {
        case .error:
            return .error
        case .warn:
            return .warn
        case .info:
            return .info
        case .debug:
            return .debug
        case .trace:
            return .trace
        }
    }
}

public func initLogger(bundleIdentifier: String, level: LogLevel) {
    ZKsyncSSOFFI.initAppleLogger(
        bundleIdentifier: bundleIdentifier,
        level: level.ffi
    )
}
