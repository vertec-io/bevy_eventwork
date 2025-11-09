# eventwork_websockets

[![Crates.io](https://img.shields.io/crates/v/eventwork_websockets)](https://crates.io/crates/eventwork_websockets)
[![Docs.rs](https://docs.rs/eventwork_websockets/badge.svg)](https://docs.rs/eventwork_websockets)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/jamescarterbell/bevy_eventwork)

WebSocket transport provider for [bevy_eventwork](https://github.com/jamescarterbell/bevy_eventwork) with full WASM and native support.

## Supported Platforms

- WASM
- Windows
- Linux
- Mac

## Features

- ✅ **WASM Support** - Works in web browsers
- ✅ **Native Support** - Works on Linux, Windows, macOS
- ✅ **Async Runtime** - Uses `async-std` for cross-platform compatibility
- ✅ **Drop-in Replacement** - Easy to switch from TCP to WebSockets

## Getting Started

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
bevy = "0.17"
eventwork = "1.1"
eventwork_websockets = "1.1"
serde = { version = "1.0", features = ["derive"] }
```

**Important**: Bevy 0.17 requires Rust 1.88.0 (nightly). Create `rust-toolchain.toml`:
```toml
[toolchain]
channel = "nightly"
```

## Version Compatibility

| eventwork_websockets | eventwork | Bevy | Rust |
| :------------------: | :-------: | :--: | :--: |
| 1.1.0 | 1.1.0 | 0.17 | 1.88 (nightly) |
| 0.2.0 | 0.9.0 | 0.16 | 1.85 |

### Basic Usage

```rust
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use eventwork::{AppNetworkMessage, EventworkPlugin, EventworkRuntime};
use eventwork_websockets::{WebSocketProvider, NetworkSettings};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the EventworkPlugin with WebSocketProvider
        .add_plugins(EventworkPlugin::<WebSocketProvider, bevy::tasks::TaskPool>::default())
        // Configure network settings
        .insert_resource(NetworkSettings::default())
        // Set up the async runtime
        .insert_resource(EventworkRuntime(
            TaskPoolBuilder::new().num_threads(2).build()
        ))
        // Register your messages
        .listen_for_message::<YourMessage, WebSocketProvider>()
        .run();
}
```

### Network Settings

Configure the WebSocket connection:

```rust
use eventwork_websockets::NetworkSettings;

// Default settings (localhost:3000)
app.insert_resource(NetworkSettings::default());

// Custom settings
app.insert_resource(NetworkSettings {
    ip: "127.0.0.1".to_string(),
    port: 8080,
});
```

## Examples

Check out the [examples directory](./examples) for complete working examples:

- **`server.rs`** - WebSocket chat server
- **`client.rs`** - WebSocket chat client with Bevy UI

Run the examples:
```bash
# Terminal 1 - Start the server
cargo run --example server -p eventwork_websockets

# Terminal 2 - Start a client
cargo run --example client -p eventwork_websockets
```

## WASM Compilation

To compile for WASM:

```bash
# Add the WASM target
rustup target add wasm32-unknown-unknown

# Build for WASM
cargo build --target wasm32-unknown-unknown --example client -p eventwork_websockets
```

No special features or configuration needed - it just works! ✨

## Version Compatibility

| eventwork_websockets | eventwork | Bevy |
| :------------------: | :-------: | :--: |
|         0.2          |    0.9    | 0.16 |
|         0.1          |    0.8    | 0.13 |

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
