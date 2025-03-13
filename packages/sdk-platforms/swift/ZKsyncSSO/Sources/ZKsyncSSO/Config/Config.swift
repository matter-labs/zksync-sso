import Foundation
import ZKsyncSSOFFI

public struct Config {
    let inner: ZKsyncSSOFFI.Config

    public init(
        contracts: PasskeyContracts,
        nodeUrl: String
    ) {
        self.inner = .init(contracts: contracts.inner, nodeUrl: nodeUrl)
    }

    public static var `default`: Self {
        let innerDefault = ZKsyncSSOFFI.Config.default
        return Self(
            contracts: PasskeyContracts.init(inner: innerDefault.contracts),
            nodeUrl: innerDefault.nodeUrl
        )
    }
}

extension ZKsyncSSOFFI.Config {
    static var `default`: Self {
        guard let configUrl = Bundle.module.url(forResource: "config", withExtension: "json") else {
            fatalError("config url couldn't be read")
        }

        guard let data = try? Data(contentsOf: configUrl) else {
            fatalError("\(configUrl) data couldn't be read")
        }

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase

        guard let config = try? decoder.decode(Self.self, from: data) else {
            fatalError("config.json data coulnd't be read")
        }

        return config
    }
}
