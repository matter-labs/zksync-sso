import Foundation

public struct UIError: Error {
    public let message: String

    public init(message: String) {
        self.message = message
    }

    public init(from error: Error) {
        self.message = error.localizedDescription
    }
}
