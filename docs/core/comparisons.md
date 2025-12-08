# Comparisons

How does Eventwork compare to other networking solutions for Bevy and Rust applications?

---

## At a Glance

| Feature | Eventwork | bevy_renet | Matchbox | WebSocket (tungstenite) |
|---------|-----------|------------|----------|-------------------------|
| Transport Agnostic | ✅ | ❌ (UDP) | ❌ (WebRTC) | ❌ (WS only) |
| ECS Component Sync | ✅ | ❌ | ❌ | ❌ |
| Leptos Integration | ✅ | ❌ | ❌ | ❌ |
| Built-in DevTools | ✅ | ❌ | ❌ | ❌ |
| Browser Support | ✅ | ❌ | ✅ | ✅ |
| Game Prediction | ❌ | ✅ | ✅ | ❌ |
| Matchmaking | ❌ | ❌ | ✅ | ❌ |

---

## bevy_renet

[bevy_renet](https://github.com/lucaspoffo/renet) is a UDP-based networking library focused on game development with client-side prediction.

### When to Choose bevy_renet

- Building competitive multiplayer games
- Need client-side prediction and lag compensation
- UDP-based transport is acceptable

### When to Choose Eventwork

- Need browser client support (WebSocket)
- Building dashboards or control interfaces
- Want automatic ECS component synchronization
- Need transport flexibility (TCP, WebSocket, custom)

---

## Matchbox

[Matchbox](https://github.com/johanhelsing/matchbox) provides peer-to-peer WebRTC networking with matchmaking support.

### When to Choose Matchbox

- Building peer-to-peer games
- Need NAT traversal
- Want built-in matchmaking

### When to Choose Eventwork

- Have an authoritative server architecture
- Need component-level synchronization
- Building industrial/enterprise applications
- Want Leptos web client integration

---

## Raw WebSocket Libraries

Libraries like [tungstenite](https://github.com/snapview/tungstenite-rs) or [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite) provide low-level WebSocket support.

### When to Use Raw WebSockets

- Need complete protocol control
- Building custom messaging protocols
- Simple request/response patterns

### When to Choose Eventwork

- Want type-safe message definitions
- Need automatic serialization (bincode)
- Want ECS-native component synchronization
- Building reactive web clients

---

## Feature Comparison Table

| Capability | Eventwork | bevy_renet | Matchbox |
|------------|-----------|------------|----------|
| **Transport** |
| TCP | ✅ | ❌ | ❌ |
| UDP | ❌ | ✅ | ❌ |
| WebSocket | ✅ | ❌ | ❌ |
| WebRTC | ❌ | ❌ | ✅ |
| Custom | ✅ | ❌ | ❌ |
| **Architecture** |
| Client-Server | ✅ | ✅ | ✅ |
| Peer-to-Peer | ❌ | ❌ | ✅ |
| Authoritative Server | ✅ | ✅ | ❌ |
| **Features** |
| Component Sync | ✅ | ❌ | ❌ |
| Change Detection | ✅ | ❌ | ❌ |
| Mutation Auth | ✅ | ❌ | ❌ |
| Rate Limiting | ✅ | ❌ | ❌ |
| Conflation | ✅ | ❌ | ❌ |
| **Clients** |
| Bevy | ✅ | ✅ | ✅ |
| Leptos (WASM) | ✅ | ❌ | ❌ |
| Browser (JS) | ✅ | ❌ | ✅ |

---

## Summary

**Choose Eventwork when:**
- Building real-time dashboards and control interfaces
- Need automatic ECS component synchronization
- Want Leptos/WASM web client support
- Require transport flexibility

**Choose alternatives when:**
- Building competitive multiplayer games (bevy_renet)
- Need peer-to-peer with matchmaking (Matchbox)
- Need complete protocol control (raw WebSockets)
