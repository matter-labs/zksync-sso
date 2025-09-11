# zkSync SSO ERC-4337

A Rust workspace for integrating zkSync SSO with ERC-4337 account abstraction.

## Workspace Structure

This workspace contains the following crates:

- **`zksync-sso-erc4337-core`**: Core functionality and business logic
- **`zksync-sso-erc4337-web-ffi`**: WebAssembly FFI bindings for web applications
- **`zksync-sso-erc4337-ffi`**: FFI bindings for mobile platforms (iOS/Android)
- **`zksync-sso-erc4337-cli`**: Command-line interface

## Dependencies

- **Core**: All crates depend on the core crate for shared functionality
- **Web FFI**: Provides WebAssembly bindings using `wasm-bindgen`
- **FFI**: Provides mobile platform bindings using `uniffi`
- **CLI**: Provides command-line interface using `clap`

## Building

To build the entire workspace:

```bash
cargo build
```

To build a specific crate:

```bash
cargo build -p zksync-sso-erc4337-core
cargo build -p zksync-sso-erc4337-web-ffi
cargo build -p zksync-sso-erc4337-ffi
cargo build -p zksync-sso-erc4337-cli
```

## Testing

To run tests for all crates:

```bash
cargo test
```

To run tests for a specific crate:

```bash
cargo test -p zksync-sso-erc4337-core
```

## Usage

### CLI

```bash
# Initialize the client
cargo run -p zksync-sso-erc4337-cli -- init

# Show configuration
cargo run -p zksync-sso-erc4337-cli -- config

# Test connection
cargo run -p zksync-sso-erc4337-cli -- test
```

### Core Library

```rust
use zksync_sso_erc4337_core::{ZkSyncSsoErc4337Client, ZkSyncSsoErc4337Config};

let config = ZkSyncSsoErc4337Config::new(
    "https://sepolia.era.zksync.dev".to_string(),
    "https://bundler.example.com".to_string(),
    "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789".to_string(),
    "0x9406Cc6185a346906296840746125a0E44976454".to_string(),
);

let client = ZkSyncSsoErc4337Client::new(config);
client.initialize().await?;
```

## License

Licensed under the Apache License, Version 2.0.
