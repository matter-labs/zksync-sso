import Foundation
import ZKsyncSSOFFI

public struct Config {
    let inner: ZKsyncSSOFFI.Config

    public init(
        contracts: PasskeyContracts,
        nodeUrl: String,
        deployWallet: DeployWallet
    ) {
        self.inner = .init(
            contracts: contracts.inner,
            nodeUrl: nodeUrl,
            deployWallet: deployWallet
        )
    }

    public static var `default`: Self {
        let innerDefault = ZKsyncSSOFFI.Config.default
        return Self(
            contracts: PasskeyContracts(inner: innerDefault.contracts),
            nodeUrl: innerDefault.nodeUrl,
            deployWallet: DeployWallet(
                privateKeyHex: innerDefault.deployWallet.privateKeyHex
            )
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

        guard let config = try? decoder.decode(Self.self, from: data) else {
            fatalError("config.json data coulnd't be read")
        }

        return config
    }
}
