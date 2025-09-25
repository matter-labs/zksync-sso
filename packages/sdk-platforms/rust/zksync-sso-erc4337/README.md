# zkSync SSO ERC-4337

ZKsync SSO SDK compatible with ERC-4337 account abstraction.

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

## License

Licensed under the Apache License, Version 2.0.
