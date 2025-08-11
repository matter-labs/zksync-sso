import Foundation

struct EOASigner {
    let privateKeyHex: String
    let address: String

    init(privateKeyHex: String, address: String) {
        self.privateKeyHex = privateKeyHex
        self.address = address
    }
}

extension EOASigner {
    static let accountOwner = EOASigner(
        privateKeyHex: "0x5de4111afa1a4b94908f83103c3a57e0c3c9e9da2dd5a02a84e9fde30d7e96c3",
        address: "0x6a34ea49c29bf7cce95f51e7f0f419831ad5dbc6"
    )
    
    static let sessionOwner = EOASigner(
        privateKeyHex: "0x6954ddb21936036ccad688e2770846f15380a721bfab26c6e531e25b35cb5971",
        address: "0x9BbC92a33F193174bf6Cc09c4b4055500d972479"
    )
}
