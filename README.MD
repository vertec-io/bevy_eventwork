# bevy_eventwork

> A modular, event-driven networking solution for [Bevy](https://bevyengine.org/) games. Connect multiple Bevy instances with ease using a flexible, transport-agnostic architecture.

Forked from the excellent [`bevy_spicy_networking`](https://crates.io/crates/bevy_spicy_networking), with significant improvements for modularity, performance, and ease of use.

[![Crates.io](https://img.shields.io/crates/v/bevy_eventwork)](https://crates.io/crates/bevy_eventwork)
[![Docs.rs](https://docs.rs/bevy_eventwork/badge.svg)](https://docs.rs/bevy_eventwork)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/jamescarterbell/bevy_eventwork)

## Contents

- [Features](#features)
- [Documentation](#documentation)
  - [Quickstart](#quickstart)
- [Examples](#examples)
- [Bevy Version Compatibility](#bevy-version-compatibility)
- [Supported Platforms](#supported-platforms)
- [Transport Providers](#transport-providers)
- [Workspace Crates](#workspace-crates)
- [Roadmap](#roadmap)
- [Contributing](#contributing)

## Features

- **🔌 Transport Agnostic**: Use TCP, WebSockets, or implement your own transport layer
- **🌐 Cross-Platform**: Works on Linux, Windows, macOS, and WASM
- **⚡ Event-Driven**: Integrates seamlessly with Bevy's ECS event system
- **🔄 Request/Response**: Built-in support for request/response messaging patterns
- **📦 Modular**: Lightweight core with optional features and transport providers
- **🎯 Type-Safe**: Strongly typed messages with compile-time guarantees
- **🚀 Async Runtime Agnostic**: Works with any async runtime (Bevy TaskPool, Tokio, etc.)

## Documentation

📚 **[Online Documentation](https://docs.rs/bevy_eventwork)** - Complete API reference

You can also build the documentation locally:
```bash
cargo doc -p eventwork --open
```

### Quickstart

#### 1. Add Dependencies

```toml
[dependencies]
bevy = "0.16"
eventwork = "0.9"
serde = { version = "1.0", features = ["derive"] }

# Choose a transport provider:
# eventwork_websockets = "0.2"  # For WebSocket support (WASM + Native)
```

#### 2. Define Your Messages

```rust
use serde::{Serialize, Deserialize};
use eventwork::NetworkMessage;

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    user: String,
    message: String,
}

impl NetworkMessage for ChatMessage {
    const NAME: &'static str = "example:ChatMessage";
}
```

#### 3. Set Up Your App

```rust
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use eventwork::{AppNetworkMessage, EventworkPlugin, EventworkRuntime, NetworkProvider};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the networking plugin with your chosen transport
        .add_plugins(EventworkPlugin::<YourTransportProvider, bevy::tasks::TaskPool>::default())
        // Set up the async runtime
        .insert_resource(EventworkRuntime(
            TaskPoolBuilder::new().num_threads(2).build()
        ))
        // Register messages to listen for
        .listen_for_message::<ChatMessage, YourTransportProvider>()
        // Add your systems
        .add_systems(Update, handle_chat_messages)
        .run();
}
```

#### 4. Handle Incoming Messages

```rust
use eventwork::NetworkData;

fn handle_chat_messages(
    mut messages: EventReader<NetworkData<ChatMessage>>,
) {
    for message in messages.read() {
        println!("{}: {}", message.inner.user, message.inner.message);
    }
}
```

#### 5. Send Messages

```rust
use eventwork::Network;

fn send_chat_message(
    net: Res<Network<YourTransportProvider>>,
) {
    let message = ChatMessage {
        user: "Player1".to_string(),
        message: "Hello, world!".to_string(),
    };

    // Send to a specific connection
    net.send_message(connection_id, message).ok();

    // Or broadcast to all connections
    net.broadcast_message(message).ok();
}
```

## Examples

Check out the [examples directory](https://github.com/jamescarterbell/bevy_eventwork/tree/master/crates/eventwork/examples) for complete working examples:

- **`server.rs`** - A chat server that broadcasts messages to all connected clients
- **`client.rs`** - A graphical chat client with Bevy UI

Run the examples:
```bash
# Terminal 1 - Start the server
cargo run --example server -p eventwork

# Terminal 2 - Start a client
cargo run --example client -p eventwork
```

For WebSocket examples, see the [`eventwork_websockets` crate](./crates/eventwork_websockets).

## Bevy Version Compatibility

| bevy_eventwork | Bevy | Rust |
| :------------: | :--: | :--: |
|      0.9       | 0.16 | 1.85 |
|      0.8       | 0.12 | 1.76 |
|      0.7       | 0.8  | 1.70 |

**Note**: Versions not compatible with the latest Bevy are in maintenance mode and will only receive critical bug fixes.

## Supported Platforms

| Platform | Status | Notes |
| :------: | :----: | :---: |
| **Linux** | ✅ Fully Supported | Tested on Ubuntu, Debian, Arch |
| **Windows** | ✅ Fully Supported | Tested on Windows 10/11 |
| **macOS** | ⚠️ Should Work | Not regularly tested - community feedback welcome! |
| **WASM** | ✅ Supported | Requires WebSocket transport provider |

**WASM Support**: Use the [`eventwork_websockets`](./crates/eventwork_websockets) transport provider for full WASM compatibility.

## Transport Providers

bevy_eventwork uses a modular transport system. Choose the provider that fits your needs:

| Provider | Platforms | WASM | Status | Crate |
| :------: | :-------: | :--: | :----: | :---: |
| **TCP** | Linux, Windows, macOS | ❌ | ✅ Included | Built-in |
| **WebSocket** | Linux, Windows, macOS, WASM | ✅ | ✅ Available | [`eventwork_websockets`](./crates/eventwork_websockets) |
| **Memory** | All | ✅ | 🧪 Testing | [`eventwork_memory`](./crates/eventwork_memory) |

### Implementing Custom Transports

You can implement your own transport layer by implementing the `NetworkProvider` trait. See the [documentation](https://docs.rs/bevy_eventwork/latest/bevy_eventwork/trait.NetworkProvider.html) for details.

## Workspace Crates

This repository is organized as a Cargo workspace with multiple crates:

### Core Crates

- **[`eventwork`](./crates/eventwork)** - The main networking library
- **[`eventwork_common`](./crates/eventwork_common)** - Shared types and utilities
- **[`eventwork_macros`](./crates/eventwork_macros)** - Procedural macros

### Transport Providers

- **[`eventwork_websockets`](./crates/eventwork_websockets)** - WebSocket transport (WASM + Native)
- **[`eventwork_memory`](./crates/eventwork_memory)** - In-memory transport for testing

## Roadmap

### Current Focus
- ✅ Bevy 0.16 support
- ✅ Rust 2024 edition
- ✅ Improved documentation
- ✅ WebSocket transport provider

### Future Plans
- 🔄 Message type ID optimization (reduce bandwidth by using numeric IDs instead of strings)
- 🔄 Enhanced request/response patterns
- 🔄 Connection pooling and load balancing
- 🔄 Built-in encryption support
- 🔄 Metrics and monitoring tools
- 🔄 RPC-style function calls

### Community Contributions Welcome!
- Additional transport providers (QUIC, UDP, etc.)
- Performance optimizations
- More examples and tutorials
- Platform-specific testing (especially macOS)

## Contributing

Contributions are welcome! Here's how you can help:

### Ways to Contribute

- 🐛 **Report bugs** - Open an issue with reproduction steps
- 💡 **Suggest features** - Share your ideas for improvements
- 📝 **Improve documentation** - Help make the docs clearer
- 🔧 **Submit PRs** - Fix bugs or implement features
- 🧪 **Test on different platforms** - Especially macOS!
- 📦 **Create transport providers** - Implement new network protocols

### Development Setup

```bash
# Clone the repository
git clone https://github.com/jamescarterbell/bevy_eventwork.git
cd bevy_eventwork

# Build the workspace
cargo build --workspace --all-features

# Run tests
cargo test --workspace

# Run examples
cargo run --example server -p eventwork
cargo run --example client -p eventwork
```

### Guidelines

- Follow Rust's standard style guidelines (`cargo fmt`)
- Ensure all tests pass (`cargo test`)
- Add tests for new features
- Update documentation for API changes
- Keep PRs focused on a single change

### Getting Help

- 💬 **Discord**: Find us on the [Bevy Discord](https://discord.gg/bevy) - look for `@SirCarter`
- 📖 **Documentation**: Check the [online docs](https://docs.rs/bevy_eventwork)
- 🐛 **Issues**: Browse [existing issues](https://github.com/jamescarterbell/bevy_eventwork/issues) or open a new one

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

---

**Built with ❤️ for the Bevy community**
