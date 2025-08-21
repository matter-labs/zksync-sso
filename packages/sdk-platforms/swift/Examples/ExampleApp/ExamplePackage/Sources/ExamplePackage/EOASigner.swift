import Foundation
import ZKsyncSSOIntegration

struct EOASigner: Signer {
    let address: String
    let privateKeyHex: String

    init(address: String, privateKeyHex: String) {
        self.privateKeyHex = privateKeyHex
        self.address = address
    }
}

extension EOASigner {
    static let accountOwner = EOASigner(
        address: "0x90F79bf6EB2c4f870365E785982E1f101E93b906",
        privateKeyHex: "0x5de4111afa1a4b94908f83103c3a57e0c3c9e9da2dd5a02a84e9fde30d7e96c3"  // Correct address for this private key
    )
    
    static let sessionOwner = EOASigner(
        address: "0x9BbC92a33F193174bf6Cc09c4b4055500d972479",
        privateKeyHex: "0x6954ddb21936036ccad688e2770846f15380a721bfab26c6e531e25b35cb5971"
    )
    
    // Additional signer matching the debug test second session
    static let debugSessionOwner = EOASigner(
        address: "0x90F79bf6EB2c4f870365E785982E1f101E93b906",
        privateKeyHex: "0x5de4111afa1a4b94908f83103c3a57e0c3c9e9da2dd5a02a84e9fde30d7e96c3"
    )
}
