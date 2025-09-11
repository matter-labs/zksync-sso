use alloy::{contract::private::Provider, primitives::Address};

pub async fn is_smart_account_deployed<P>(
    provider: &P,
    sender_address: Address,
) -> eyre::Result<bool>
where
    P: Provider,
{
    let contract_code = provider.get_code_at(sender_address).await?;

    if contract_code.len() > 2 {
        return Ok(true);
    }

    Ok(false)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use alloy::{
//         network::EthereumWallet,
//         providers::ProviderBuilder,
//         signers::local::{coins_bip39::English, mnemonic::MnemonicBuilder},
//     };

//     const MNEMONIC_PHRASE: &str =
//         "test test test test test test test test test test test junk";

//     #[tokio::test]
//     async fn test_is_smart_account_deployed() -> eyre::Result<()> {
//         let config = crate::config::Config::local();
//         let chain = crate::chain::Chain::ETHEREUM_SEPOLIA_V07;
//         let entry_point_config = chain.entry_point_config();
//         // let chain_id = chain.id.eip155_chain_id();
//         let entry_point_address = entry_point_config.address();

//         let (owner_address, _local_signer, provider) = {
//             let phrase = MNEMONIC_PHRASE;
//             let index: u32 = 0;
//             let local_signer = MnemonicBuilder::<English>::default()
//                 .phrase(phrase)
//                 .index(index)?
//                 .build()?;
//             let ethereum_wallet = EthereumWallet::from(local_signer.clone());
//             let rpc_url_string = config.endpoints.rpc.base_url;
//             let rpc_url: reqwest::Url = rpc_url_string.parse()?;
//             let provider = ProviderBuilder::new()
//                 .with_recommended_fillers()
//                 .wallet(ethereum_wallet.clone())
//                 .on_http(rpc_url);
//             let owner = ethereum_wallet.clone().default_signer();
//             let owner_address = owner.address();
//             eyre::Ok((owner_address, local_signer, provider))
//         }?;

//         use crate::smart_accounts::simple_account::factory::FactoryAddress;
//         let simple_account_factory_address = FactoryAddress::local_v07();
//         use crate::entry_point::get_sender_address::get_sender_address_v07;

//         use crate::smart_accounts::simple_account::create_account::SimpleAccountCreate;

//         let factory_data_call = SimpleAccountCreate::new_u64(owner_address, 0);
//         let factory_data_value = factory_data_call.encode();

//         let sender_address = get_sender_address_v07(
//             &provider,
//             simple_account_factory_address.into(),
//             factory_data_value.into(),
//             entry_point_address,
//         )
//         .await?;

//         let is_deployed =
//             is_smart_account_deployed(&provider, sender_address).await?;

//         println!("is_deployed: {:?}", is_deployed);

//         Ok(())
//     }
// }
