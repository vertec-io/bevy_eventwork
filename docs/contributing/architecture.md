# Architecture Reference

This document provides architectural reference for bevy_eventwork contributors.

---

## System Overview

`eventwork_sync` is a reflection-driven synchronization middleware that enables real-time bidirectional synchronization between a Bevy ECS server and arbitrary clients.

### Components

- **Server-side** (`eventwork_sync`): Bevy plugin that tracks component changes and broadcasts updates
- **Client-side** (`eventwork_client`): Leptos-based reactive client library with automatic subscription management

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Bevy ECS Server                              │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  EventworkSyncPlugin                                           │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐ │ │
│  │  │ Component    │  │ Change       │  │ Subscription         │ │ │
│  │  │ Registry     │  │ Detection    │  │ Manager              │ │ │
│  │  │              │  │              │  │                      │ │ │
│  │  │ - Position   │  │ - Added      │  │ - Client 1: Position │ │ │
│  │  │ - Velocity   │  │ - Changed    │  │ - Client 2: Velocity │ │ │
│  │  │ - EntityName │  │ - Removed    │  │ - Client 3: All      │ │ │
│  │  └──────────────┘  └──────────────┘  └──────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                  │                                   │
│                                  ▼                                   │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  Broadcast System                                              │ │
│  │  - Batches component changes per frame                         │ │
│  │  - Sends SyncBatch messages to subscribed clients              │ │
│  └────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  │ WebSocket (NetworkPacket)
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Leptos Web Client                            │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  SyncProvider (WebSocket Handler)                              │ │
│  │  - Receives NetworkPacket messages                             │ │
│  │  - Deserializes SyncServerMessage                              │ │
│  │  - Updates component_data (raw bytes)                          │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                  │                                   │
│                                  ▼                                   │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  SyncContext                                                   │ │
│  │  - component_data: RwSignal<HashMap<(u64, String), Vec<u8>>>   │ │
│  │  - subscribe_component<T>() → RwSignal<HashMap<u64, T>>        │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                  │                                   │
│                                  ▼                                   │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  UI Components                                                 │ │
│  │  - Application views with synced data                          │ │
│  │  - DevTools: hierarchical entity inspector                     │ │
│  └────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Wire Protocol

All messages are wrapped in `NetworkPacket` for transport:

```rust
pub struct NetworkPacket {
    pub type_name: String,      // Fully qualified type name
    pub schema_hash: u64,       // Schema version
    pub data: Vec<u8>,          // Bincode-serialized message
}
```

### Client → Server Messages

```rust
pub enum SyncClientMessage {
    Subscription(SubscriptionRequest),
    Unsubscribe(UnsubscribeRequest),
    Mutate(MutateComponent),
    Query(QueryRequest),
    QueryCancel(QueryCancel),
}
```

### Server → Client Messages

```rust
pub enum SyncServerMessage {
    SyncBatch(SyncBatch),           // Component updates
    MutationResponse(MutationResponse),
    QueryResponse(QueryResponse),
}
```

---

## Key Data Flows

### Subscription Flow

1. Client calls `use_sync_component::<Position>()`
2. Hook sends `SubscriptionRequest` to server
3. Server adds subscription to `SubscriptionManager`
4. Server sends initial snapshot (all matching entities)
5. Server sends incremental updates on changes

### Mutation Flow

1. Client calls `ctx.mutate::<Position>(entity_id, new_value)`
2. Client sends `MutateComponent` to server
3. Server validates via `MutationAuthorizer`
4. If approved: component updated, change broadcasted
5. Server sends `MutationResponse` to client

### Change Detection Flow

1. Bevy's change detection marks components as changed
2. `broadcast_component_changes` runs each frame
3. Changes collected into `HashMap<ConnectionId, Vec<SyncItem>>`
4. One `SyncBatch` sent per subscribed connection

---

## Design Philosophy

### Industrial Applications First

The system is designed for:
- **High frame rates** (30-60+ FPS updates)
- **High throughput** (thousands of component updates per second)
- **Low latency** (sub-frame response times)
- **Deterministic behavior** (predictable performance under load)

### Per-Frame Batching

All component changes are batched per frame:
- One `SyncBatch` message per connection per frame
- At 60 FPS: 60 messages/second per client
- Efficient for high-update-rate applications

---

## Related Documentation

- [Server Setup Guide](../guides/server-setup.md)
- [Subscriptions Guide](../guides/subscriptions.md)
- [Performance Reference](./performance.md)

---

**Last Updated**: 2025-12-07

