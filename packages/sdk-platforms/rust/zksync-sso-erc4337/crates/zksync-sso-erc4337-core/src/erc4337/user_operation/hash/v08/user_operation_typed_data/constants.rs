use alloy::{
    primitives::address,
    sol,
    sol_types::{Eip712Domain, SolStruct, eip712_domain},
};
use serde::Serialize;

sol! {
    #[allow(missing_docs)]
    #[derive(Serialize)]
    struct Person {
        string name;
        address wallet;
    }

    #[allow(missing_docs)]
    #[derive(Serialize)]
    struct Mail {
        Person from;
        Person to;
        string contents;
    }
}

pub struct HashPayload<T: SolStruct> {
    pub domain: Eip712Domain,
    pub message: T,
}

pub fn basic() -> HashPayload<Mail> {
    HashPayload {
        domain: eip712_domain! {
            name: "Ether Mail",
            version: "1",
            chain_id: 1,
            verifying_contract: address!("0x0000000000000000000000000000000000000000"),
        },
        message: Mail {
            from: Person {
                name: "Cow".to_string(),
                wallet: address!("0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"),
            },
            to: Person {
                name: "Bob".to_string(),
                wallet: address!("0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"),
            },
            contents: "Hello, Bob!".to_string(),
        },
    }
}
