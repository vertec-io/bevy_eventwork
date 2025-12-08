# Core Eventwork Guide

This guide covers the core `eventwork` crate for basic networking in Bevy.

## Overview

`eventwork` provides event-driven networking for Bevy applications:

- **Transport Agnostic** - Use TCP, WebSocket, or custom transports
- **Type-Safe Messages** - Strongly typed with compile-time guarantees
- **Event-Driven** - Integrates with Bevy's ECS event system
- **Zero Boilerplate** - Automatic message registration (v0.10+)

## Quick Start

### 1. Define Messages

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    user: String,
    message: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct PlayerPosition {
    x: f32,
    y: f32,
}
```

### 2. Set Up the Server

```rust
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use eventwork::{
    EventworkPlugin, EventworkRuntime, Network, NetworkEvent,
    AppNetworkMessage, tcp::{TcpProvider, NetworkSettings},
};

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(EventworkPlugin::<TcpProvider, bevy::tasks::TaskPool>::default())
        .insert_resource(EventworkRuntime(
            TaskPoolBuilder::new().num_threads(2).build()
        ))
        .insert_resource(NetworkSettings::default())
        // Register messages
        .register_network_message::<ChatMessage, TcpProvider>()
        .register_network_message::<PlayerPosition, TcpProvider>()
        // Add systems
        .add_systems(Startup, start_server)
        .add_systems(Update, (handle_connections, handle_messages))
        .run();
}

fn start_server(net: Res<Network<TcpProvider>>) {
    net.listen("127.0.0.1:3000".parse().unwrap());
    info!("Server listening on 127.0.0.1:3000");
}
```

### 3. Handle Connections

```rust
fn handle_connections(
    mut events: MessageReader<NetworkEvent>,
    net: Res<Network<TcpProvider>>,
) {
    for event in events.read() {
        match event {
            NetworkEvent::Connected(conn_id) => {
                info!("Client connected: {:?}", conn_id);
                // Send welcome message
                net.send(*conn_id, ChatMessage {
                    user: "Server".to_string(),
                    message: "Welcome!".to_string(),
                });
            }
            NetworkEvent::Disconnected(conn_id) => {
                info!("Client disconnected: {:?}", conn_id);
            }
            NetworkEvent::Error(err) => {
                error!("Network error: {:?}", err);
            }
        }
    }
}
```

### 4. Handle Messages

```rust
use eventwork::NetworkData;

fn handle_messages(
    mut messages: MessageReader<NetworkData<ChatMessage>>,
    net: Res<Network<TcpProvider>>,
) {
    for msg in messages.read() {
        info!("{}: {}", msg.user, msg.message);
        
        // Broadcast to all clients
        net.broadcast(ChatMessage {
            user: msg.user.clone(),
            message: msg.message.clone(),
        });
    }
}
```

### 5. Set Up the Client

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EventworkPlugin::<TcpProvider, bevy::tasks::TaskPool>::default())
        .insert_resource(EventworkRuntime(
            TaskPoolBuilder::new().num_threads(2).build()
        ))
        .insert_resource(NetworkSettings::default())
        .register_network_message::<ChatMessage, TcpProvider>()
        .add_systems(Startup, connect_to_server)
        .add_systems(Update, handle_chat_messages)
        .run();
}

fn connect_to_server(net: Res<Network<TcpProvider>>) {
    net.connect("127.0.0.1:3000".parse().unwrap());
}

fn handle_chat_messages(
    mut messages: MessageReader<NetworkData<ChatMessage>>,
) {
    for msg in messages.read() {
        println!("{}: {}", msg.user, msg.message);
    }
}
```

## Key Concepts

### NetworkData<T>

Wraps received messages with connection info:

```rust
fn handle_messages(mut messages: MessageReader<NetworkData<ChatMessage>>) {
    for msg in messages.read() {
        let source = msg.source();  // ConnectionId
        let user = &msg.user;       // Access via Deref
    }
}
```

### Network Resource

The main interface for sending messages:

```rust
fn send_messages(net: Res<Network<TcpProvider>>) {
    // Send to specific client
    net.send(conn_id, message.clone());
    
    // Broadcast to all
    net.broadcast(message);
}
```

## Next Steps

- [WebSocket Transport](../../crates/eventwork_websockets/README.md) - For WASM support
- [Server Sync](./eventwork-sync.md) - Automatic component synchronization
- [Sending Messages Guide](../guides/sending-messages.md) - Advanced patterns

