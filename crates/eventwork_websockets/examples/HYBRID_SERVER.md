# Hybrid Server Example

## Overview

The `hybrid_server` example demonstrates how to run **both TCP and WebSocket protocols simultaneously** in a single Bevy server, allowing clients from either protocol to connect and share a common chat room.

## Architecture

The hybrid server uses Bevy's resource system to maintain two separate `Network<T>` resources:

- `Network<TcpProvider>` - Handles TCP connections on port **3030**
- `Network<WebSocketProvider>` - Handles WebSocket connections on port **8081**

Both networks are registered with the same message types, and a bridge system forwards messages between the two protocols.

## How It Works

1. **Dual Plugin Registration**: Both `EventworkPlugin<TcpProvider>` and `EventworkPlugin<WebSocketProvider>` are added to the same Bevy app
2. **Message Registration**: All message types are registered for both providers
3. **Unified Connection Registry**: A custom resource tracks connections from both protocols
4. **Message Bridging**: Systems listen for messages from each protocol and broadcast them to all clients on both protocols

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
2. The bridge system detects the message
3. The message is broadcast to:
   - All clients on the **same protocol** (except the sender)
   - All clients on the **other protocol**

This creates a unified chat room where TCP and WebSocket clients can communicate seamlessly!

## Current Limitations

### ConnectionId Collision

Each `Network<T>` resource maintains its own connection counter, so TCP and WebSocket clients can have the same ConnectionId (e.g., both start at ID=1). This causes issues when trying to send messages because the ConnectionId is not unique across protocols.

**Workaround**: The current implementation uses the `UnifiedConnectionRegistry` to track which ConnectionIds belong to which protocol, preventing cross-protocol confusion.

### Future Improvements

1. **Unified ConnectionId System**: Implement a global connection counter that ensures unique IDs across all protocols
2. **Connection Metadata**: Add protocol type to ConnectionId or create a wrapper type
3. **Optimized Broadcasting**: Use `broadcast_except()` method to avoid duplicate code
4. **Error Handling**: Better handling of send failures and disconnections

## Code Structure

- `setup_networking()` - Starts both TCP and WebSocket servers
- `handle_tcp_connection_events()` - Tracks TCP client connections/disconnections
- `handle_ws_connection_events()` - Tracks WebSocket client connections/disconnections
- `handle_tcp_messages()` - Receives TCP messages and broadcasts to all clients
- `handle_ws_messages()` - Receives WebSocket messages and broadcasts to all clients
- `UnifiedConnectionRegistry` - Maintains separate lists of TCP and WebSocket connections

## Benefits

✅ **Protocol Flexibility**: Clients can choose their preferred protocol  
✅ **Unified Chat Room**: All clients see messages from all other clients regardless of protocol  
✅ **Zero Code Duplication**: Message types are defined once and work for both protocols  
✅ **Hybrid Schema Hash**: Cross-protocol communication works even with different module paths  
✅ **Scalable**: Can easily add more protocols (e.g., UDP, QUIC) using the same pattern  

## Example Output

```
📡 TCP server listening on 127.0.0.1:3030
🌐 WebSocket server listening on 127.0.0.1:8081
🚀 Hybrid server started! Accepting both TCP and WebSocket connections.

📡 TCP client connected: Connection with ID=1
📡 TCP connection added: Connection with ID=1 (Total TCP: 1, WS: 0)

🌐 WebSocket client connected: Connection with ID=1
🌐 WebSocket connection added: Connection with ID=1 (Total TCP: 1, WS: 1)

📡 Received TCP message from Connection with ID=1: Hello from TCP!
🌐 Received WebSocket message from Connection with ID=1: Hello from WebSocket!
```

## Conclusion

The hybrid server demonstrates that **bevy_eventwork's architecture is flexible enough to support multiple protocols simultaneously**. While there are some rough edges with ConnectionId management, the core concept works and provides a powerful foundation for building cross-protocol networked applications!

