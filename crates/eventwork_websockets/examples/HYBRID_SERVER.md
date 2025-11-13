# Hybrid Server Example

## Overview

The `hybrid_server` example demonstrates how to run **both TCP and WebSocket protocols simultaneously** in a single Bevy server, allowing clients from either protocol to connect and share a common chat room.

This example showcases the **scheduled outbound message pattern**, which provides complete decoupling of game logic from network infrastructure.

## Architecture

The hybrid server uses Bevy's resource system to maintain two separate `Network<T>` resources:

- `Network<TcpProvider>` - Handles TCP connections on port **3030**
- `Network<WebSocketProvider>` - Handles WebSocket connections on port **8081**

Both networks are registered with the same message types, and a **two-stage relay system** handles message broadcasting.

## Scheduled Outbound Message Pattern

This example uses the **`outbound_scheduled`** feature flag, which implements a two-stage message relay system:

### Stage 1: Game Logic (GameLogic SystemSet)
Game logic systems read incoming messages and write `OutboundMessage<T>` events. They have **zero dependencies** on `Network` resources.

```rust
fn handle_messages(
    mut new_messages: MessageReader<NetworkData<UserChatMessage>>,
    mut outbound: MessageWriter<OutboundMessage<NewChatMessage>>,
) {
    for message in new_messages.read() {
        // Determine protocol using provider_name field
        let provider = message.provider_name();  // "TCP" or "WebSocket"

        // Create broadcast message
        let broadcast_message = NewChatMessage {
            name: format!("{}-{}", provider, message.source()),
            message: message.message.clone(),
        };

        // Write outbound - relay system handles the rest!
        outbound.write(OutboundMessage {
            name: "chat".to_string(),
            message: broadcast_message,
            for_client: None,  // None = broadcast to all
        });
    }
}
```

### Stage 2: Network Relay (NetworkRelay SystemSet)
The relay system runs **after** game logic and broadcasts all queued messages to the actual network providers.

```rust
fn relay_hybrid_outbound(
    mut outbound_messages: MessageReader<OutboundMessage<NewChatMessage>>,
    tcp_net: Res<Network<TcpProvider>>,
    ws_net: Res<Network<WebSocketProvider>>,
) {
    for notification in outbound_messages.read() {
        match &notification.for_client {
            Some(client) => {
                // Send to specific client on both providers
                tcp_net.send(*client, notification.message.clone()).ok();
                ws_net.send(*client, notification.message.clone()).ok();
            }
            None => {
                // Broadcast to all clients on both providers
                tcp_net.broadcast(notification.message.clone());
                ws_net.broadcast(notification.message.clone());
            }
        }
    }
}
```

### System Set Ordering

```rust
app.configure_sets(Update, (
    GameLogic,
    NetworkRelay.after(GameLogic),
));
```

This ensures:
- ✅ All game logic completes before messages are sent
- ✅ Messages are sent at a deterministic point in the frame
- ✅ Can use `.apply_deferred()` before NetworkRelay to sync world state

## Provider Identification

The `NetworkData<T>` type includes a `provider_name` field that identifies which protocol the message came from:

```rust
let provider = message.provider_name();  // "TCP" or "WebSocket"
```

This allows game logic to determine the protocol **without** needing access to `Network` resources, achieving complete decoupling.

## How It Works

1. **Dual Plugin Registration**: Both `EventworkPlugin<TcpProvider>` and `EventworkPlugin<WebSocketProvider>` are added to the same Bevy app
2. **Message Registration**: All message types are registered for both providers
3. **Unified Connection Tracking**: Connection events are handled for both protocols
4. **Two-Stage Message Relay**:
   - Game logic writes `OutboundMessage<T>` events (GameLogic set)
   - Relay system broadcasts to all providers (NetworkRelay set)

## Running the Example

### Start the Hybrid Server

```bash
cargo run --example hybrid_server --package eventwork_websockets
```

The server will listen on:
- 📡 **TCP**: `127.0.0.1:3030`
- 🌐 **WebSocket**: `127.0.0.1:8081`

### Connect TCP Clients

```bash
cargo run -p eventwork --example client
```

### Connect WebSocket Clients

**Bevy Client:**
```bash
cargo run --example client --package eventwork_websockets
```

**Leptos WASM Client:**
```bash
cd crates/eventwork_websockets/leptos_client_example
trunk serve --port 8082
# Open http://127.0.0.1:8082 in your browser
```

## Message Flow

When a client sends a message:

1. The message is received by the appropriate `Network<T>` resource
2. The message is written to Bevy's global `MessageWriter<NetworkData<UserChatMessage>>` with the provider name
3. Game logic reads the message and writes an `OutboundMessage<NewChatMessage>` (GameLogic set)
4. The relay system reads all outbound messages and broadcasts them to both TCP and WebSocket clients (NetworkRelay set)

This creates a unified chat room where TCP and WebSocket clients can communicate seamlessly!

## Benefits of the Scheduled Pattern

✅ **Complete Decoupling**: Game logic has zero dependencies on `Network` resources
✅ **Provider Identification**: Can determine TCP vs WebSocket using `message.provider_name()`
✅ **Determinism**: All messages sent at the same point in the frame
✅ **Testability**: Game logic can be tested without network infrastructure
✅ **Flexibility**: Easy to add new protocols without changing game logic
✅ **Batching**: Can apply `.apply_deferred()` before relay to sync world state

## Alternative: Immediate Pattern

The `outbound_immediate` feature flag (default) provides a simpler approach where messages are sent immediately when `OutboundMessage<T>` is written. This is useful for:
- Simple applications that don't need deterministic timing
- Prototyping and quick development
- Cases where you want messages sent as soon as possible

To use the immediate pattern, change the feature flag in `Cargo.toml`:
```toml
eventwork = { path = "../eventwork", features = ["tcp", "outbound_immediate"], default-features = false }
```

## Code Structure

- `setup_networking()` - Starts both TCP and WebSocket servers
- `handle_connection_events()` - Unified handler for connections/disconnections from both protocols
- `handle_messages()` - Game logic that reads incoming messages and writes outbound messages (GameLogic set)
- `relay_hybrid_outbound()` - Relay system that broadcasts outbound messages to both providers (NetworkRelay set)

## Summary of Benefits

✅ **Protocol Flexibility**: Clients can choose their preferred protocol (TCP or WebSocket)
✅ **Unified Chat Room**: All clients see messages from all other clients regardless of protocol
✅ **Zero Code Duplication**: Message types are defined once and work for both protocols
✅ **Hybrid Schema Hash**: Cross-protocol communication works even with different module paths
✅ **Complete Decoupling**: Game logic has no dependencies on network infrastructure
✅ **Provider Identification**: Determine protocol using `message.provider_name()`
✅ **Deterministic Messaging**: All messages sent at the same point in the frame
✅ **Scalable**: Can easily add more protocols (e.g., UDP, QUIC) using the same pattern

## Example Output

```
📡 TCP server listening on 127.0.0.1:3030
🌐 WebSocket server listening on 127.0.0.1:8081
🚀 Hybrid server started! Accepting both TCP and WebSocket connections.

📡 TCP client connected: Connection with ID=1
🌐 WebSocket client connected: Connection with ID=1

📡 Received TCP message from 1: Hello from TCP!
🌐 Received WebSocket message from 1: Hello from WebSocket!
```

## Conclusion

The hybrid server demonstrates that **bevy_eventwork's architecture is flexible enough to support multiple protocols simultaneously** with a clean, decoupled design. The scheduled outbound message pattern provides:

- **Clean separation** between game logic and network infrastructure
- **Deterministic behavior** with predictable message timing
- **Easy testing** since game logic has no network dependencies
- **Scalability** to add new protocols without changing existing code

This pattern is ideal for production applications that need robust, maintainable networking code!

