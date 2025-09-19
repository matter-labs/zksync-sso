use crate::{
    chain::id::ChainId, erc4337::entry_point::version::EntryPointVersion,
};

pub struct EntryPointConfig {
    pub chain_id: ChainId,
    pub version: EntryPointVersion,
}

// impl EntryPointConfig {
//     pub const V07_MAINNET: EntryPointConfig = EntryPointConfig {
//         chain_id: ChainId::ETHEREUM_MAINNET,
//         version: EntryPointVersion::V07,
//     };

//     pub const V07_SEPOLIA: EntryPointConfig = EntryPointConfig {
//         chain_id: ChainId::ETHEREUM_SEPOLIA,
//         version: EntryPointVersion::V07,
//     };

//     pub const V07_LOCAL_FOUNDRY_SEPOLIA: EntryPointConfig = EntryPointConfig {
//         chain_id: ChainId::LOCAL_FOUNDRY_ETHEREUM_SEPOLIA,
//         version: EntryPointVersion::V07,
//     };

//     pub fn address(&self) -> EntryPointAddress {
//         // assuming that the entrypoint address is the same for all chains, so
//         // not matching based on `chain_id` (anymore)
//         match self.version {
//             EntryPointVersion::V06 => {
//                 EntryPointAddress::new(ENTRYPOINT_ADDRESS_V06)
//             }
//             EntryPointVersion::V07 => {
//                 EntryPointAddress::new(ENTRYPOINT_ADDRESS_V07)
//             }
//         }
//     }

//     pub fn type_string(&self) -> String {
//         self.version.type_string()
//     }
// }
