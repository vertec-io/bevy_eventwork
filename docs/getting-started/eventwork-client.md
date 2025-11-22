# Getting Started with eventwork_client

**eventwork_client** is a reactive Leptos library for building web UIs that synchronize with Bevy ECS servers via eventwork_sync.

**Time**: 30-45 minutes  
**Difficulty**: Intermediate  
**Prerequisites**: Basic Leptos knowledge, eventwork_sync server running

---

## Overview

eventwork_client provides:
- **Reactive hooks** - Subscribe to components with automatic updates
- **Type-safe** - Compile-time type checking
- **Focus retention** - Editable fields that don't lose focus on server updates
- **DevTools** - Built-in component inspector
- **Automatic subscription management** - Subscribe on mount, unsubscribe on unmount

---

## Installation

Add to your `Cargo.toml`:

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

---

## Quick Start

### Step 1: Define Shared Types

Create a shared types crate or module that both server and client can use:

```rust
use serde::{Serialize, Deserialize};

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}
```

**Note**: Client types don't need `Component` or `Reflect` - just `Serialize` and `Deserialize`.

### Step 2: Implement SyncComponent

Use the `impl_sync_component!` macro to make your types work with eventwork_client:

```rust
use eventwork_client::impl_sync_component;

impl_sync_component!(Position);
impl_sync_component!(Velocity);
```

This macro implements the `SyncComponent` trait, which provides type information for serialization.

### Step 3: Set Up the Client Registry

Create a registry that maps type names to deserializers:

```rust
use leptos::prelude::*;
use eventwork_client::{SyncProvider, ClientRegistryBuilder};

#[component]
pub fn App() -> impl IntoView {
    let registry = ClientRegistryBuilder::new()
        .register::<Position>()
        .register::<Velocity>()
        .build();
    
    view! {
        <SyncProvider
            url="ws://localhost:8082"
            registry=registry
        >
            <GameView/>
        </SyncProvider>
    }
}
```

### Step 4: Subscribe to Components

Use the `use_sync_component` hook to subscribe and display data:

```rust
use eventwork_client::use_sync_component;

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

### Step 5: Create index.html

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8"/>
    <title>My Game Client</title>
</head>
<body></body>
</html>
```

### Step 6: Build and Run

```bash
trunk serve --port 8080
```

Open `http://localhost:8080` in your browser. You should see the synchronized entities!

---

## Editable Fields

To allow users to edit component values:

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
            <label>
                "Y: "
                <SyncFieldInput
                    entity_id=entity_id
                    field_accessor=|pos: &Position| pos.y
                    field_mutator=|pos: &Position, new_y: f32| {
                        Position { x: pos.x, y: new_y }
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

---

## DevTools

eventwork_client includes built-in DevTools for inspecting entities and components:

```rust
use eventwork_client::DevTools;

#[component]
fn App() -> impl IntoView {
    let registry = ClientRegistryBuilder::new()
        .register::<Position>()
        .build();
    
    view! {
        <SyncProvider url="ws://localhost:8082" registry=registry>
            <GameView/>
            <DevTools/>  // Add DevTools
        </SyncProvider>
    }
}
```

Press the DevTools button to inspect entities, view component values, and edit fields in real-time.

---

## Next Steps

- **[Full Stack Example](./full-stack-example.md)** - Complete client-server application
- **[Mutations Guide](../guides/mutations.md)** - Advanced mutation patterns
- **[DevTools Guide](../guides/devtools.md)** - DevTools features and customization
- **[Type Registry Guide](../guides/type-registry.md)** - Advanced registry patterns

---

## Complete Example

See `crates/eventwork_client/examples/basic_client/` for a complete working example.

**Run it**:
```bash
# Terminal 1: Start server
cargo run -p eventwork_client --example basic_server

# Terminal 2: Start client
cd crates/eventwork_client/examples/basic_client
trunk serve --port 8080
```

---

**Last Updated**: 2025-11-22  
**eventwork_client Version**: 0.1  
**Leptos Version**: 0.8

