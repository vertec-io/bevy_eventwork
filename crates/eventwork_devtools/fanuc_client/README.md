# FANUC Robot Control Client

A Leptos-based web client for controlling a simulated FANUC robot using bevy_eventwork's sync functionality.

## Prerequisites

Install Trunk (WASM build tool):
```bash
cargo install trunk
```

Add WASM target:
```bash
rustup target add wasm32-unknown-unknown
```

## Running

### 1. Start the FANUC Server

In one terminal:
```bash
cd /home/apino/dev/bevy_eventwork
cargo run -p eventwork_sync --example fanuc_server --features runtime
```

You should see:
```
INFO fanuc_server: FANUC robot initialized
INFO fanuc_server: FANUC server listening on 127.0.0.1:8082
```

### 2. Start the Web Client

In another terminal:
```bash
cd /home/apino/dev/bevy_eventwork/crates/eventwork_devtools/fanuc_client
trunk serve
```

You should see:
```
INFO 📦 starting build
INFO 🌎 serving at http://127.0.0.1:8083
```

### 3. Open in Browser

Navigate to: `http://127.0.0.1:8083`

1. Enter host: `127.0.0.1`
2. Enter port: `8082`
3. Click "Connect"
4. Use the jog controls to move the robot
5. Click "Show DevTools" to inspect the ECS state

## Features

- **Robot Status Display**: Shows servo ready, TP enabled, in motion, and error status
- **Position Display**: Shows X, Y, Z (mm) and W, P, R (degrees)
- **Jog Controls**: Buttons to jog each axis in positive/negative direction
- **DevTools Integration**: Toggle side-by-side DevTools panel to inspect ECS entities and components

## Architecture

This client uses:
- **Leptos 0.8**: Reactive web framework compiled to WASM
- **eventwork_sync**: Client-side sync protocol for Bevy ECS
- **eventwork_devtools**: Reusable DevTools widget
- **fanuc_shared**: Shared component types between server and client
- **Tailwind CSS**: Styling via CDN

## Development

Build for release:
```bash
trunk build --release
```

The output will be in the `dist/` directory.

## Next Steps

- Wire jog buttons to send `MutateComponent<JogCommand>` messages
- Wire displays to show real-time data from subscriptions
- Add 3D visualization of robot
- Implement forward/inverse kinematics

