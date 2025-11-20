# Eventwork Sync Architecture

## Overview

`eventwork_sync` is a reflection-driven synchronization middleware that enables real-time bidirectional synchronization between a Bevy ECS server and arbitrary clients (web, native, mobile) over eventwork.

The system consists of two main components:
- **Server-side** (`eventwork_sync`): Bevy plugin that tracks component changes and broadcasts updates
- **Client-side** (`eventwork_client`): Leptos-based reactive client library with automatic subscription management

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
                                  │
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
│  │  SyncContext (Meteorite Pattern)                               │ │
│  │  ┌──────────────────────────────────────────────────────────┐ │ │
│  │  │ component_data: RwSignal<HashMap<(u64, String), Vec<u8>>>│ │ │
│  │  │                                                            │ │ │
│  │  │ Key: (entity_id, component_name)                          │ │ │
│  │  │ Value: Raw serialized bytes                               │ │ │
│  │  └──────────────────────────────────────────────────────────┘ │ │
│  │                                                                  │ │
│  │  ┌──────────────────────────────────────────────────────────┐ │ │
│  │  │ subscribe_component<T>() → RwSignal<HashMap<u64, T>>     │ │ │
│  │  │                                                            │ │ │
│  │  │ Creates Effect that:                                      │ │ │
│  │  │ 1. Watches component_data for changes                     │ │ │
│  │  │ 2. Filters by component name                              │ │ │
│  │  │ 3. Deserializes bytes → T                                 │ │ │
│  │  │ 4. Updates typed signal                                   │ │ │
│  │  └──────────────────────────────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                  │                                   │
│                                  ▼                                   │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  UI Components                                                 │ │
│  │  - EntityList: displays entities with Position/Velocity        │ │
│  │  - DevTools: hierarchical entity inspector                     │ │
│  └────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Wire Protocol

All messages are wrapped in `NetworkPacket` for transport:

```rust
pub struct NetworkPacket {
    pub type_name: String,      // Fully qualified type name
    pub schema_hash: u64,        // Schema version (currently unused)
    pub data: Vec<u8>,           // Bincode-serialized message
}
```

### Client → Server Messages

All client messages must be wrapped in `SyncClientMessage`:

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

All server messages are wrapped in `SyncServerMessage`:

```rust
pub enum SyncServerMessage {
    SyncBatch(SyncBatch),
    MutationResponse(MutationResponse),
    QueryResponse(QueryResponse),
}
```

## The Meteorite Pattern

The client uses the "Meteorite pattern" for handling WebSocket messages:

### Traditional Approach (❌ Problematic)
```rust
// Deserialize immediately in WebSocket handler
let message: SyncServerMessage = deserialize(bytes)?;
// Now what? How do we route this to the right component?
```

**Problems:**
- WebSocket handler needs to know about all component types at compile time
- Tight coupling between network layer and UI components
- Difficult to add new component types dynamically

### Meteorite Pattern (✅ Solution)

**Two-tier storage:**
1. **Raw bytes** stored in type-agnostic signal
2. **Effects** watch bytes and deserialize to typed signals

```rust
// Tier 1: Store raw bytes (type-agnostic)
component_data: RwSignal<HashMap<(u64, String), Vec<u8>>>

// Tier 2: Effect watches bytes and deserializes (type-aware)
Effect::new(move |_| {
    let data_map = component_data.get();
    let mut typed_map = HashMap::new();
    
    for ((entity_id, comp_name), bytes) in data_map.iter() {
        if comp_name == &component_name_str {
            match registry.deserialize::<T>(comp_name, bytes) {
                Ok(component) => { typed_map.insert(*entity_id, component); }
                Err(err) => { /* log error */ }
            }
        }
    }
    
    signal.set(typed_map);
});
```

**Benefits:**
- WebSocket handler is completely type-agnostic
- Components subscribe to what they need
- Easy to add new component types
- Automatic reactivity through Leptos Effects

## Subscription Flow

### 1. Component Mounts
```rust
let positions = ctx.subscribe_component::<Position>();
```

### 2. Subscription Request Sent
```rust
// Wait for WebSocket to open
Effect::new(move |_| {
    if ready_state.get() == ConnectionReadyState::Open {
        // Wrap in SyncClientMessage
        let message = SyncClientMessage::Subscription(SubscriptionRequest {
            subscription_id: 1,
            component_type: "Position".to_string(),
            entity: None,  // Subscribe to all entities
        });
        
        // Serialize and send
        let bytes = bincode::encode_to_vec(&message)?;
        send(&bytes);
    }
});
```

### 3. Server Processes Subscription
```rust
// Server receives and deserializes
let client_msg: SyncClientMessage = deserialize(bytes)?;

match client_msg {
    SyncClientMessage::Subscription(req) => {
        // Store subscription
        subscriptions.insert(conn_id, req);
        
        // Send snapshot of current state
        send_snapshot(conn_id, &req.component_type);
    }
}
```

### 4. Server Sends Updates
```rust
// Every frame, broadcast changes
for (conn_id, subscription) in subscriptions.iter() {
    let batch = SyncBatch {
        items: vec![
            ComponentUpdate::Update {
                entity: 123,
                component_name: "Position".to_string(),
                value: bincode::encode(&Position { x: 1.0, y: 2.0 }),
            },
        ],
    };
    
    let server_msg = SyncServerMessage::SyncBatch(batch);
    send(conn_id, &server_msg);
}
```

### 5. Client Receives and Updates
```rust
// SyncProvider receives NetworkPacket
let packet: NetworkPacket = codec.decode(bytes)?;
let server_msg: SyncServerMessage = deserialize(&packet.data)?;

match server_msg {
    SyncServerMessage::SyncBatch(batch) => {
        for item in batch.items {
            match item {
                ComponentUpdate::Update { entity, component_name, value } => {
                    // Store raw bytes
                    component_data.update(|map| {
                        map.insert((entity, component_name), value);
                    });
                }
            }
        }
    }
}

// Effect automatically triggers and deserializes
// UI components automatically re-render
```

## Performance Characteristics

### Server-Side
- **Change Detection**: O(n) where n = number of changed components per frame
- **Subscription Matching**: O(m) where m = number of active subscriptions
- **Serialization**: O(k) where k = size of component data
- **Memory**: Subscriptions stored in HashMap, O(1) lookup

### Client-Side
- **Deserialization**: Only when component_data changes (reactive)
- **Memory**: Raw bytes + deserialized objects (double storage)
- **Reactivity**: Leptos Effects provide fine-grained updates

### Network
- **Protocol**: Binary (bincode) - compact and fast
- **Batching**: Multiple updates per frame batched into single message
- **Compression**: Not currently implemented (future optimization)

## Common Patterns

### Subscribe to All Entities
```rust
let positions = ctx.subscribe_component::<Position>();
// Returns RwSignal<HashMap<u64, Position>>
```

### Subscribe to Specific Entity
```rust
let position = ctx.subscribe_component_for_entity::<Position>(entity_id);
// Returns RwSignal<Option<Position>>
```

### Display Entities in UI
```rust
#[component]
fn EntityList() -> impl IntoView {
    let ctx = use_context::<SyncContext>().unwrap();
    let positions = ctx.subscribe_component::<Position>();
    
    view! {
        <For
            each=move || positions.get().into_iter().collect::<Vec<_>>()
            key=|(id, _)| *id
            children=move |(id, pos)| {
                view! {
                    <div>{format!("Entity {}: ({}, {})", id, pos.x, pos.y)}</div>
                }
            }
        />
    }
}
```

## Next Steps

See [EVENTWORK_SYNC_PERFORMANCE.md](./EVENTWORK_SYNC_PERFORMANCE.md) for performance analysis and best practices.

