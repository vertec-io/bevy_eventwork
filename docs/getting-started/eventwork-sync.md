# Getting Started with eventwork_sync

**eventwork_sync** is a server-side Bevy plugin that automatically synchronizes ECS components to connected clients using bincode serialization.

**Time**: 30-45 minutes
**Difficulty**: Intermediate
**Prerequisites**: Basic Bevy knowledge, eventwork setup

---

## Overview

eventwork_sync provides:
- **Automatic component synchronization** - Components are automatically sent to subscribed clients
- **Bincode serialization** - Fast binary serialization, no reflection required
- **Opt-in per component** - Only components you register are synchronized
- **Mutation support** - Clients can request component changes (with authorization)
- **Configurable** - Control update rates, conflation, and more

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy = "0.17"
eventwork = "1.1"
eventwork_sync = "0.1"
eventwork_websockets = "1.1"
serde = { version = "1.0", features = ["derive"] }
```

---

## Quick Start

### Step 1: Create a Shared Crate

The recommended pattern is to create a shared crate that both server and client can use. This allows type definitions to be shared without requiring the client to depend on Bevy.

**Create `shared_types/Cargo.toml`**:

```toml
[package]
name = "shared_types"
version = "0.1.0"
edition = "2021"

[features]
server = ["dep:bevy"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
bevy = { version = "0.17", optional = true }
```

**Create `shared_types/src/lib.rs`**:

```rust
use serde::{Deserialize, Serialize};

// Conditionally import Bevy only when building for server
#[cfg(feature = "server")]
use bevy::prelude::*;

// Component trait is only derived when building with "server" feature
#[cfg_attr(feature = "server", derive(Component))]
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[cfg_attr(feature = "server", derive(Component))]
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}
```

**Key Points**:
- ✅ **NO Reflect trait required** - eventwork_sync uses bincode, not reflection
- ✅ **Conditional compilation** - `Component` is only derived when `server` feature is enabled
- ✅ **Client has no Bevy dependency** - Client builds without the `server` feature
- ✅ **Same types, different traits** - Server gets `Component`, client gets just `Serialize + Deserialize`

### Step 2: Set Up the Server

**Server `Cargo.toml`**:

```toml
[dependencies]
bevy = "0.17"
eventwork = "1.1"
eventwork_sync = "0.1"
eventwork_websockets = "1.1"
shared_types = { path = "../shared_types", features = ["server"] }
```

**Server `main.rs`**:

```rust
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use eventwork::{EventworkPlugin, EventworkRuntime, NetworkSettings};
use eventwork_sync::{EventworkSyncPlugin, AppEventworkSyncExt};
use eventwork_websockets::WebSocketProvider;
use shared_types::{Position, Velocity};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);

    // Add eventwork networking
    app.add_plugins(EventworkPlugin::<WebSocketProvider, bevy::tasks::TaskPool>::default());
    app.insert_resource(EventworkRuntime(
        TaskPoolBuilder::new().num_threads(2).build()
    ));
    app.insert_resource(NetworkSettings::default());

    // Add eventwork_sync plugin
    app.add_plugins(EventworkSyncPlugin::<WebSocketProvider>::default());

    // Register components for synchronization
    app.sync_component::<Position>(None);
    app.sync_component::<Velocity>(None);

    app.add_systems(Startup, setup);
    app.add_systems(Update, move_entities);

    app.run();
}
```

### Step 3: Spawn Entities

Just spawn entities normally - eventwork_sync will automatically track changes:

```rust
fn setup(mut commands: Commands) {
    // Spawn some entities with synchronized components
    commands.spawn((
        Position { x: 0.0, y: 0.0 },
        Velocity { x: 1.0, y: 0.5 },
    ));
    
    commands.spawn((
        Position { x: 10.0, y: 5.0 },
        Velocity { x: -0.5, y: 1.0 },
    ));
}

fn move_entities(
    time: Res<Time>,
    mut query: Query<(&mut Position, &Velocity)>,
) {
    for (mut pos, vel) in &mut query {
        pos.x += vel.x * time.delta_secs();
        pos.y += vel.y * time.delta_secs();
    }
}
```

### Step 4: Start the Server

```rust
use eventwork::AppNetworkMessage;

fn setup(mut commands: Commands) {
    // ... spawn entities ...
    
    // Start listening for connections
    commands.listen("127.0.0.1:8082");
}
```

That's it! Your server is now synchronizing Position and Velocity components to any connected clients.

---

## Configuration

### Sync Settings

Configure global sync behavior:

```rust
use eventwork_sync::SyncSettings;

app.insert_resource(SyncSettings {
    // Limit updates to 30 Hz (30 updates per second)
    max_update_rate_hz: Some(30.0),
    
    // Enable message conflation (only send latest update)
    enable_message_conflation: true,
});
```

### Per-Component Configuration

Configure individual components:

```rust
use eventwork_sync::ComponentSyncConfig;

app.sync_component::<Position>(Some(ComponentSyncConfig {
    // Component-specific settings (currently reserved for future use)
}));
```

---

## How It Works

1. **Registration**: When you call `app.sync_component::<T>()`, eventwork_sync registers the component type
2. **Change Detection**: Bevy's change detection tracks when components are added, modified, or removed
3. **Subscription**: Clients send subscription requests for specific component types
4. **Synchronization**: Changes are automatically serialized and sent to subscribed clients
5. **Conflation**: If enabled, multiple updates to the same entity+component are conflated (only latest sent)
6. **Rate Limiting**: Updates are throttled to the configured max_update_rate_hz

---

## Next Steps

- **[eventwork_client Getting Started](./eventwork-client.md)** - Build a client to display this data
- **[Component Registration Guide](../guides/component-registration.md)** - Advanced registration patterns
- **[Mutation Authorization](../guides/authorization.md)** - Control client mutations
- **[Performance Tuning](../guides/performance-tuning.md)** - Optimize for your use case

---

## Complete Example

See `crates/eventwork_sync/examples/basic_sync_server.rs` for a complete working example.

**Run it**:
```bash
cargo run -p eventwork_sync --example basic_sync_server --features runtime
```

---

**Last Updated**: 2025-11-22  
**eventwork_sync Version**: 0.1  
**Bevy Version**: 0.17.2

