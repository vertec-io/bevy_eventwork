---
title: API Reference
---
# API Reference

Detailed API documentation for the bevy_eventwork ecosystem.

## Online Documentation

For the most up-to-date API documentation, see the docs.rs pages:

- **[eventwork](https://docs.rs/eventwork)** - Core networking library
- **[eventwork_common](https://docs.rs/eventwork_common)** - Shared types
- **[eventwork_sync](https://docs.rs/eventwork_sync)** - Server-side sync
- **[eventwork_client](https://docs.rs/eventwork_client)** - Leptos client
- **[eventwork_websockets](https://docs.rs/eventwork_websockets)** - WebSocket transport
- **[eventwork_macros](https://docs.rs/eventwork_macros)** - Procedural macros

## Contents

| Reference | Description |
|-----------|-------------|
| [Hooks Reference](./hooks-reference.md) | All eventwork_client hooks |
| [Control Plugin](./control-plugin.md) | ExclusiveControlPlugin API |

## Quick Reference

### eventwork_client Hooks

| Hook | Purpose |
|------|---------|
| `use_sync_component::<T>()` | Subscribe to component updates |
| `use_sync_component_store::<T>()` | Fine-grained reactive store |
| `use_sync_component_write::<T>()` | Write mutations to server |
| `use_connection_status()` | Monitor connection state |
| `use_sync_context()` | Access raw sync context |

### eventwork_sync Extension Trait

| Method | Purpose |
|--------|---------|
| `app.sync_component::<T>(settings)` | Register component for sync |
| `app.register_network_message::<T, P>()` | Register message type |

### eventwork Network Resource

| Method | Purpose |
|--------|---------|
| `net.send(conn_id, msg)` | Send to specific connection |
| `net.broadcast(msg)` | Send to all connections |
| `net.disconnect(conn_id)` | Disconnect a client |

## Building Local Documentation

Generate documentation locally:

```bash
# All crates
cargo doc --workspace --open

# Specific crate
cargo doc -p eventwork_client --open
```

## Related Documentation

- [Getting Started](../getting-started/) - Quick start guides
- [Guides](../guides/) - In-depth tutorials
- [Architecture](../architecture/) - System design

