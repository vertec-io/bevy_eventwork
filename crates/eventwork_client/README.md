# eventwork_client

Reactive Leptos client library for building web UIs that synchronize with Bevy ECS servers.

[![Crates.io](https://img.shields.io/crates/v/eventwork_client.svg)](https://crates.io/crates/eventwork_client)
[![Documentation](https://docs.rs/eventwork_client/badge.svg)](https://docs.rs/eventwork_client)
[![License](https://img.shields.io/crates/l/eventwork_client.svg)](https://github.com/vertec-io/bevy_eventwork/blob/main/LICENSE)

---

## Overview

**eventwork_client** is a reactive Leptos library for building web UIs that display and edit ECS components synchronized from Bevy servers via `eventwork_sync`. It's designed for control panels, dashboards, and web-based tools for robotics, industrial automation, and multiplayer games.

### Key Features

- ✅ **Reactive Hooks** - Subscribe to components with automatic updates
- ✅ **Type-Safe** - Compile-time type checking with Rust's type system
- ✅ **Focus Retention** - Editable fields that don't lose focus on server updates
- ✅ **DevTools** - Built-in component inspector for debugging
- ✅ **Automatic Subscription Management** - Subscribe on mount, unsubscribe on unmount
- ✅ **Bevy-Free** - No Bevy dependency, runs in WASM/browser

---

## Quick Start

### Installation

```toml
[dependencies]
leptos = "0.8"
eventwork_client = "0.1"
serde = { version = "1.0", features = ["derive"] }
```

Install Trunk for building WASM:

```bash
cargo install trunk
rustup target add wasm32-unknown-unknown
```

### Shared Crate Pattern (Recommended)

Use the same shared crate as your server. See [eventwork_sync README](../eventwork_sync/README.md) for how to create it.

**Client `Cargo.toml`**:
```toml
[dependencies]
leptos = "0.8"
eventwork_client = "0.1"
# Import shared types WITHOUT the "server" feature
shared_types = { path = "../shared_types" }
```

**Key Points**:
- ✅ **NO "server" feature** - Client builds without Bevy dependency
- ✅ **Same types as server** - Guaranteed type compatibility
- ✅ **WASM-compatible** - No Bevy means it compiles to WASM
- ✅ **Automatic trait implementation** - `SyncComponent` is automatically implemented for all `Serialize + Deserialize` types

### Basic Usage

```rust
use leptos::prelude::*;
use eventwork_client::{
    SyncProvider, use_sync_component, ClientRegistryBuilder
};
use shared_types::Position;

// SyncComponent is automatically implemented - no manual implementation needed!

#[component]
pub fn App() -> impl IntoView {
    let registry = ClientRegistryBuilder::new()
        .register::<Position>()
        .build();

    view! {
        <SyncProvider url="ws://localhost:8082" registry=registry>
            <GameView/>
        </SyncProvider>
    }
}

#[component]
fn GameView() -> impl IntoView {
    // Automatically subscribes to Position components
    let positions = use_sync_component::<Position>();

    view! {
        <div class="game-view">
            <h1>"Entities"</h1>
            <For
                each=move || {
                    positions.get()
                        .iter()
                        .map(|(id, pos)| (*id, pos.clone()))
                        .collect::<Vec<_>>()
                }
                key=|(id, _)| *id
                let:item
            >
                {
                    let (entity_id, position) = item;
                    view! {
                        <div class="entity">
                            "Entity " {entity_id} ": "
                            "x=" {position.x} ", y=" {position.y}
                        </div>
                    }
                }
            </For>
        </div>
    }
}
```

### About impl_sync_component!

The `impl_sync_component!` macro is **required** for types to work with eventwork_client hooks:

```rust
impl_sync_component!(Position);
```

**What it does**:
- Implements the `SyncComponent` trait for your type
- Provides `component_name()` method that returns the short type name
- Enables `use_sync_component::<T>()` to subscribe to the correct component type

**Why it's needed**:
- eventwork_sync identifies components by their short type name (e.g., "Position")
- The macro extracts this name using `std::any::type_name::<T>()`
- This is different from eventwork core's blanket `EventworkMessage` implementation
- Both serve different purposes and are both necessary

### Editable Fields

```rust
use eventwork_client::SyncFieldInput;

#[component]
fn PositionEditor(entity_id: u64) -> impl IntoView {
    view! {
        <div class="editor">
            <label>
                "X: "
                <SyncFieldInput
                    entity_id=entity_id
                    field_accessor=|pos: &Position| pos.x
                    field_mutator=|pos: &Position, new_x: f32| {
                        Position { x: new_x, y: pos.y }
                    }
                    input_type="number"
                />
            </label>
        </div>
    }
}
```

**Features**:
- ✅ **Focus retention** - Input doesn't lose focus when server updates arrive
- ✅ **Enter to apply** - Press Enter to send mutation to server
- ✅ **Blur to revert** - Click away to discard changes and revert to server value

### DevTools

```rust
use eventwork_client::DevTools;

view! {
    <SyncProvider url="ws://localhost:8082" registry=registry>
        <GameView/>
        <DevTools/>  // Add DevTools
    </SyncProvider>
}
```

---

## Build and Run

Create `index.html`:

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8"/>
    <title>My Client</title>
</head>
<body></body>
</html>
```

Build and serve:

```bash
trunk serve --port 8080
```

Open `http://localhost:8080` in your browser!

---

## Documentation

- **[Getting Started Guide](../../docs/getting-started/eventwork-client.md)** - Step-by-step tutorial
- **[API Documentation](https://docs.rs/eventwork_client)** - Complete API reference
- **[Mutations Guide](../../docs/guides/mutations.md)** - Advanced mutation patterns
- **[DevTools Guide](../../docs/guides/devtools.md)** - DevTools features
- **[Examples](./examples/)** - Working code examples

---

## Examples

See the `examples/` directory for complete working examples:

- **`basic_client/`** - Minimal getting started example
- **`devtools_demo/`** - DevTools integration example

Run an example:

```bash
# Terminal 1: Start server
cargo run -p eventwork_client --example basic_server

# Terminal 2: Start client
cd crates/eventwork_client/examples/basic_client
trunk serve --port 8080
```

---

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

---

**Part of the [bevy_eventwork](https://github.com/vertec-io/bevy_eventwork) ecosystem**

