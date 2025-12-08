# Installation

This guide covers installing bevy_eventwork and its dependencies.

## Prerequisites

### Rust Nightly

Bevy 0.17 requires Rust 1.88.0 (nightly). Create a `rust-toolchain.toml` in your project root:

```toml
[toolchain]
channel = "nightly"
```

Then run:

```bash
rustup update
```

### Bevy

bevy_eventwork 1.1.x requires Bevy 0.17:

```toml
[dependencies]
bevy = "0.17"
```

## Core Installation

Add the core eventwork crate:

```toml
[dependencies]
bevy = "0.17"
eventwork = "1.1"
serde = { version = "1.0", features = ["derive"] }
```

## Transport Providers

Choose a transport provider based on your needs:

### WebSocket (Recommended for Web)

```toml
[dependencies]
eventwork_websockets = "1.1"
```

Supports:
- ✅ Native (Linux, Windows, macOS)
- ✅ WASM (Web browsers)

### TCP (Built-in)

TCP is included in the core `eventwork` crate. No additional dependency needed.

Supports:
- ✅ Native (Linux, Windows, macOS)
- ❌ WASM (not supported)

### Memory (Testing)

```toml
[dependencies]
eventwork_memory = "1.1"
```

For in-memory testing without network overhead.

## Sync & Client

For ECS component synchronization:

### Server-Side (Bevy)

```toml
[dependencies]
eventwork_sync = "1.1"
```

### Client-Side (Leptos)

```toml
[dependencies]
eventwork_client = "1.1"
leptos = "0.8"
```

## Complete Example

### Server Cargo.toml

```toml
[package]
name = "my-server"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = "0.17"
eventwork = "1.1"
eventwork_websockets = "1.1"
eventwork_sync = "1.1"
serde = { version = "1.0", features = ["derive"] }
```

### Client Cargo.toml

```toml
[package]
name = "my-client"
version = "0.1.0"
edition = "2024"

[dependencies]
leptos = "0.8"
eventwork_client = "1.1"
serde = { version = "1.0", features = ["derive"] }
wasm-bindgen = "0.2"
```

## Verifying Installation

Create a simple test to verify everything is working:

```rust
use bevy::prelude::*;
use eventwork::{EventworkPlugin, EventworkRuntime};
use eventwork_websockets::WebSocketProvider;
use bevy::tasks::TaskPoolBuilder;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(EventworkPlugin::<WebSocketProvider, bevy::tasks::TaskPool>::default())
        .insert_resource(EventworkRuntime(
            TaskPoolBuilder::new().num_threads(2).build()
        ))
        .run();
}
```

If this compiles and runs without errors, you're ready to go!

## Next Steps

- [Core Eventwork Guide](./eventwork.md) - Learn the basics
- [Server Sync Guide](./eventwork-sync.md) - Set up component synchronization
- [Client Guide](./eventwork-client.md) - Build reactive web UIs

