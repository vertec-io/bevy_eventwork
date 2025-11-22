# Getting Started with bevy_eventwork

Welcome! This guide will help you get started with the bevy_eventwork ecosystem.

---

## 📖 Learning Path

We recommend following this learning path:

### 1. Core Networking (eventwork)
**Time**: 15-30 minutes  
**Guide**: [eventwork Getting Started](./eventwork.md)

Learn the basics of networking with Bevy using eventwork. This is the foundation for everything else.

**You'll learn**:
- Setting up a TCP server and client
- Sending and receiving messages
- Automatic message registration
- Type-safe networking

### 2. Server-Side Synchronization (eventwork_sync)
**Time**: 30-45 minutes  
**Guide**: [eventwork_sync Getting Started](./eventwork-sync.md)

Learn how to automatically synchronize ECS components from your Bevy server to clients.

**You'll learn**:
- Adding the EventworkSyncPlugin
- Registering components for sync
- Configuring sync settings
- Handling mutations

### 3. Client-Side Reactive UI (eventwork_client)
**Time**: 30-45 minutes  
**Guide**: [eventwork_client Getting Started](./eventwork-client.md)

Learn how to build reactive web UIs that display and edit synchronized data.

**You'll learn**:
- Setting up the SyncProvider
- Subscribing to components
- Displaying data reactively
- Implementing editable fields
- Using DevTools

### 4. Full Stack Application
**Time**: 45-60 minutes  
**Guide**: [Full Stack Example](./full-stack-example.md)

Put it all together by building a complete client-server application.

**You'll learn**:
- Project structure
- Shared types
- Complete server implementation
- Complete client implementation
- Running and testing

---

## 🎯 Quick Start by Use Case

### "I just want to send messages between Bevy systems"

→ Start with [eventwork Getting Started](./eventwork.md)

You only need the core `eventwork` crate. Skip the sync and client guides.

### "I want to build a web-based control panel for my Bevy app"

→ Follow the full learning path:
1. [eventwork Getting Started](./eventwork.md)
2. [eventwork_sync Getting Started](./eventwork-sync.md)
3. [eventwork_client Getting Started](./eventwork-client.md)
4. [Full Stack Example](./full-stack-example.md)

### "I want to build a multiplayer game"

→ Start with [eventwork Getting Started](./eventwork.md), then:
- Read the [Performance Tuning Guide](../guides/performance-tuning.md)
- Check out the [Multiplayer Game Example](../examples/multiplayer-game.md)

You may not need eventwork_sync/eventwork_client for a game - consider using eventwork directly for more control.

### "I'm migrating from Meteorite"

→ Read the [Meteorite Migration Guide](../migration/meteorite-to-eventwork.md) first, then follow the full learning path.

---

## 📋 Prerequisites

### For All Guides

- **Rust**: Stable or nightly (nightly recommended for Bevy)
- **Bevy**: 0.17 or later
- **Basic Bevy knowledge**: Understanding of ECS, systems, and plugins

### For eventwork_client Guides

- **Leptos**: 0.8
- **Trunk**: For building WASM applications
- **Basic web development knowledge**: HTML, CSS, JavaScript concepts

### Installation

**Trunk** (for client-side development):
```bash
cargo install trunk
```

**wasm32 target** (for client-side development):
```bash
rustup target add wasm32-unknown-unknown
```

---

## 🔑 Key Concepts

Before diving in, familiarize yourself with these key concepts:

### Automatic Message Registration (eventwork)

eventwork automatically registers message types - no boilerplate needed:

```rust
#[derive(Serialize, Deserialize)]
struct MyMessage {
    data: String,
}

// That's it! No manual registration required.
```

### Reflection-Driven Sync (eventwork_sync)

eventwork_sync uses Bevy's reflection system to automatically serialize and synchronize components:

```rust
#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
struct Position {
    x: f32,
    y: f32,
}

// Register for sync
app.register_sync_component::<Position>();
```

### Reactive Subscriptions (eventwork_client)

eventwork_client provides reactive hooks that automatically manage subscriptions:

```rust
// Subscribe to all Position components
let positions = use_sync_component::<Position>();

// Automatically updates when server sends changes
view! {
    <For each=move || positions.get() ...>
}
```

---

## 🚀 Ready to Start?

Choose your starting point:

- **[eventwork Getting Started](./eventwork.md)** - Core networking
- **[eventwork_sync Getting Started](./eventwork-sync.md)** - Server-side sync
- **[eventwork_client Getting Started](./eventwork-client.md)** - Client-side UI
- **[Full Stack Example](./full-stack-example.md)** - Complete application

---

## 📚 Additional Resources

- **[Architecture Overview](../architecture/system-overview.md)** - Understand how it all works
- **[User Guides](../guides/README.md)** - Task-specific how-to guides
- **[API Reference](https://docs.rs/eventwork)** - Detailed API documentation
- **[Examples](../examples/README.md)** - Real-world example applications

---

**Last Updated**: 2025-11-22  
**Difficulty**: Beginner to Intermediate  
**Estimated Time**: 2-3 hours for full learning path

