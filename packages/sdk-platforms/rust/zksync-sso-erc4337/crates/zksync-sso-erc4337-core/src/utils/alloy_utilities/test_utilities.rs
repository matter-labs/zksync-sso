pub mod config;
pub mod node_backend;
pub mod node_handle;

use crate::{
    config::contracts::Contracts,
    erc4337::{
        bundler::pimlico::client::BundlerClient,
        utils::{
            alto_test_utils::{AltoTestHelper, AltoTestHelperConfig},
            deployment_utils::deploy_contracts_default,
        },
    },
    utils::alloy_utilities::{
        ethereum_wallet_from_private_key,
        test_utilities::{
            config::TestInfraConfig,
            node_backend::{TestNodeBackend, resolve_test_node_backend},
            node_handle::TestNodeHandle,
        },
    },
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
use eyre::Context as _;
use tokio::task;
use url::Url;
use zksync_sso_zksyncos_node::instance::ZkSyncOsInstance;

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

pub async fn start_node_and_deploy_contracts()
-> eyre::Result<(Url, TestNodeHandle, AlloyProvider, Contracts, String)> {
    start_node_and_deploy_contracts_with_config(&TestInfraConfig::default())
        .await
}

pub async fn start_node_and_deploy_contracts_with_config(
    config: &TestInfraConfig,
) -> eyre::Result<(Url, TestNodeHandle, AlloyProvider, Contracts, String)> {
    let (node_url, test_node, provider, contracts, signer_private_key) =
        match resolve_test_node_backend() {
            TestNodeBackend::Anvil => start_anvil_backend(config).await,
            TestNodeBackend::ZkSyncOs => start_zksync_os_backend(config).await,
        }?;

    Ok((node_url, test_node, provider, contracts, signer_private_key))
}

async fn start_anvil_backend(
    config: &TestInfraConfig,
) -> eyre::Result<(Url, TestNodeHandle, AlloyProvider, Contracts, String)> {
    let (anvil_url, anvil_instance, provider, contracts) =
        spawn_anvil_and_deploy(config).await?;

    Ok((
        anvil_url.clone(),
        TestNodeHandle::Anvil(anvil_instance),
        provider,
        contracts,
        config.signer_private_key.to_string(),
    ))
}

async fn spawn_anvil_and_deploy(
    config: &TestInfraConfig,
) -> eyre::Result<(Url, AnvilInstance, AlloyProvider, Contracts)> {
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

    Ok((anvil_url, anvil_instance, provider, contracts))
}

async fn start_zksync_os_backend(
    config: &TestInfraConfig,
) -> eyre::Result<(Url, TestNodeHandle, AlloyProvider, Contracts, String)> {
    let zk_instance = task::spawn_blocking(ZkSyncOsInstance::spawn)
        .await
        .wrap_err("failed to spawn zksync-os-server helper")??;
    let node_url = zk_instance.rpc_url().clone();

    let provider = {
        let ethereum_wallet =
            ethereum_wallet_from_private_key(&config.signer_private_key)?;
        ProviderBuilder::new()
            .wallet(ethereum_wallet)
            .connect_http(node_url.clone())
    };

    let contracts = deploy_contracts_default(&node_url).await?;

    Ok((
        node_url,
        TestNodeHandle::ZkSyncOs(zk_instance),
        provider,
        contracts,
        config.signer_private_key.to_string(),
    ))
}

pub async fn start_node_and_deploy_contracts_and_start_bundler()
-> eyre::Result<(
    Url,
    TestNodeHandle,
    AlloyProvider,
    Contracts,
    String,
    AltoTestHelper,
    BundlerClient,
)> {
    start_node_and_deploy_contracts_and_start_bundler_with_config(
        &TestInfraConfig::default(),
    )
    .await
}

pub async fn start_node_and_deploy_contracts_and_start_bundler_with_config(
    config: &TestInfraConfig,
) -> eyre::Result<(
    Url,
    TestNodeHandle,
    AlloyProvider,
    Contracts,
    String,
    AltoTestHelper,
    BundlerClient,
)> {
    let (node_url, test_node, provider, contracts, signer_private_key) =
        start_node_and_deploy_contracts_with_config(config).await?;

    let mut alto = {
        let mut alto_cfg = match test_node {
            TestNodeHandle::ZkSyncOs(_) => {
                AltoTestHelperConfig::default_zksyncos()
            }
            TestNodeHandle::Anvil(_) => {
                AltoTestHelperConfig::default_ethereum()
            }
        };
        alto_cfg.node_url = node_url.clone();
        AltoTestHelper::new(alto_cfg)
    };

    alto.start().await?;

    let bundler_client = alto.bundler_client();

    Ok((
        node_url,
        test_node,
        provider,
        contracts,
        signer_private_key,
        alto,
        bundler_client,
    ))
}
