use crate::{
    config::contracts::Contracts,
    erc4337::{
        bundler::pimlico::client::BundlerClient,
        client::alto_test_utils::{AltoTestHelper, AltoTestHelperConfig},
        utils::deployment_utils::deploy_contracts_default,
    },
    utils::alloy_utilities::ethereum_wallet_from_private_key,
};
use alloy::{
    network::EthereumWallet,
    node_bindings::{Anvil, AnvilInstance},
    providers::{
        Identity, ProviderBuilder, RootProvider,
        fillers::{
            BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill,
            NonceFiller, WalletFiller,
        },
    },
};
use url::Url;

pub fn fork_mainnet() -> Anvil {
    let fork_url = "https://reth-ethereum.ithaca.xyz/rpc";
    let chain_id = 1337;
    Anvil::new().fork(fork_url).chain_id(chain_id)
}

type AlloyProvider = FillProvider<
    JoinFill<
        JoinFill<
            Identity,
            JoinFill<
                GasFiller,
                JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>,
            >,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider,
>;

#[derive(Debug, Clone)]
pub struct TestInfraConfig {
    pub signer_private_key: String,
}

impl Default for TestInfraConfig {
    fn default() -> Self {
        Self {
            signer_private_key: "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
                .to_string(),
        }
    }
}

pub async fn start_anvil_and_deploy_contracts()
-> eyre::Result<(Url, AnvilInstance, AlloyProvider, Contracts, String)> {
    start_anvil_and_deploy_contracts_with_config(&TestInfraConfig::default())
        .await
}

pub async fn start_anvil_and_deploy_contracts_with_config(
    config: &TestInfraConfig,
) -> eyre::Result<(Url, AnvilInstance, AlloyProvider, Contracts, String)> {
    let (anvil_url, anvil_instance) = {
        let anvil = fork_mainnet();
        let anvil_instance = anvil.try_spawn()?;
        let anvil_url = anvil_instance.endpoint_url();
        (anvil_url, anvil_instance)
    };

    let provider = {
        let ethereum_wallet =
            ethereum_wallet_from_private_key(&config.signer_private_key)?;
        ProviderBuilder::new()
            .wallet(ethereum_wallet)
            .connect_http(anvil_url.clone())
    };

    let contracts = deploy_contracts_default(&anvil_url).await?;

    Ok((
        anvil_url,
        anvil_instance,
        provider,
        contracts,
        config.signer_private_key.to_string(),
    ))
}

pub async fn start_anvil_and_deploy_contracts_and_start_bundler()
-> eyre::Result<(
    Url,
    AnvilInstance,
    AlloyProvider,
    Contracts,
    String,
    AltoTestHelper,
    BundlerClient,
)> {
    start_anvil_and_deploy_contracts_and_start_bundler_with_config(
        &TestInfraConfig::default(),
    )
    .await
}

pub async fn start_anvil_and_deploy_contracts_and_start_bundler_with_config(
    config: &TestInfraConfig,
) -> eyre::Result<(
    Url,
    AnvilInstance,
    AlloyProvider,
    Contracts,
    String,
    AltoTestHelper,
    BundlerClient,
)> {
    let (node_url, anvil_instance, provider, contracts, signer_private_key) =
        start_anvil_and_deploy_contracts_with_config(config).await?;

    let mut alto = {
        let alto_cfg = AltoTestHelperConfig {
            node_url: node_url.clone(),
            ..Default::default()
        };
        AltoTestHelper::new(alto_cfg)
    };

    alto.start().await?;

    let bundler_client = alto.bundler_client();

    Ok((
        node_url,
        anvil_instance,
        provider,
        contracts,
        signer_private_key,
        alto,
        bundler_client,
    ))
}
