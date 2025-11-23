# Bevy Eventwork Examples

This directory contains all examples for the `bevy_eventwork` project, organized by use case.

## Structure

```
examples/
├── shared/              # Shared types used by multiple examples
│   ├── basic_types/     # Types for basic example
│   ├── demo_types/      # Types for demo
│   ├── fanuc_types/     # Types for FANUC demo
│   └── fanuc_real_types/# Real FANUC RMI API types
├── basic/               # Basic client-server example
│   ├── server/          # Bevy ECS server
│   └── client/          # Leptos WASM client
├── fanuc/               # FANUC robot control example
│   ├── server/          # Bevy ECS server with FANUC simulation
│   └── client/          # Leptos WASM client
└── devtools-demo/       # DevTools demonstration
    └── server/          # Server for DevTools testing
```

## Running Examples

### Basic Example

**Server:**
```bash
cargo run -p basic_server
```

**Client:**
```bash
cd examples/basic/client
trunk serve --port 8081
```

Then open http://127.0.0.1:8081/

### FANUC Example

**Server:**
```bash
cargo run -p fanuc_server
```

**Client:**
```bash
cd examples/fanuc/client
trunk serve --port 8082
```

Then open http://127.0.0.1:8082/

### DevTools Demo

**Server:**
```bash
cargo run -p devtools_demo_server
```

Then connect with any client using the DevTools widget.

## Example Descriptions

### Basic Example
Demonstrates the core functionality of `eventwork_client`:
- WebSocket connection to Bevy ECS server
- Component synchronization (Position, Velocity, EntityName)
- Real-time updates
- DevTools integration

### FANUC Example
Shows how to use `eventwork_client` for industrial robot control:
- Real FANUC RMI API types
- Robot position and status monitoring
- Joint angle visualization
- Mutation support for robot commands

### DevTools Demo
Demonstrates the DevTools widget capabilities:
- Entity inspection
- Component viewing
- Subscription management
- Mutation testing

## Shared Types

All shared type crates follow the same pattern:
- `#[derive(Serialize, Deserialize)]` for network serialization
- `#[cfg_attr(feature = "server", derive(Component))]` for conditional Bevy integration
- Feature flag `server` enables Bevy dependency

This allows the same types to be used on both server (with Bevy) and client (WASM, without Bevy).
