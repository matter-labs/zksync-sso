use clap::Subcommand;

#[derive(Subcommand)]
pub enum InfraCommands {
    /// Deploy ERC-4337 infrastructure (EntryPoint and Factory)
    Deploy {
        /// RPC URL for deployment
        #[arg(long, default_value = "http://localhost:8545")]
        rpc_url: String,
        /// Private key for deployment (defaults to Anvil's first account)
        #[arg(long)]
        private_key: Option<String>,
    },

    /// Manage all infrastructure components
    #[command(subcommand)]
    All(AllCommands),

    /// Manage Anvil node
    #[command(subcommand)]
    Anvil(AnvilCommands),

    /// Manage bundler services
    #[command(subcommand)]
    Bundler(crate::bundler::BundlerCommands),

    /// Check status of all services
    Status,
}

#[derive(Subcommand)]
pub enum AllCommands {
    /// Start all infrastructure (Anvil + default bundler)
    Start {
        /// Sepolia fork URL (optional)
        #[arg(long)]
        fork_url: Option<String>,
        /// Bundler type (alto or rundler, defaults to rundler)
        #[arg(long, default_value = "rundler")]
        bundler: String,
    },

    /// Stop all infrastructure
    Stop,
}

#[derive(Subcommand)]
pub enum AnvilCommands {
    /// Start Anvil node
    Start {
        /// Sepolia fork URL (optional)
        #[arg(long)]
        fork_url: Option<String>,
    },

    /// Stop Anvil node
    Stop,
}
