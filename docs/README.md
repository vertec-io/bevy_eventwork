# bevy_eventwork Documentation

Welcome to the comprehensive documentation for the bevy_eventwork ecosystem!

## 🚀 Quick Start

**New to bevy_eventwork?** Start here:

1. **Core Networking** - TCP-based networking with Bevy (coming soon)
2. **Server-Side Sync** - Synchronize ECS components to clients (coming soon)
3. **Client-Side Reactive UI** - Build reactive web UIs with Leptos (coming soon)
4. **Full Stack Example** - Complete client-server application (coming soon)

---

## 📚 Documentation Structure

This documentation is organized into the following sections:

### getting-started/
Step-by-step guides to get you up and running quickly.

### architecture/
Deep dives into system architecture and design.

### guides/
How-to guides for specific tasks and features.

### api/
API reference and quick reference guides.

### examples/
Detailed walkthroughs of example applications.

### migration/
Migration guides for upgrading between versions.

### reference/
Reference materials, glossary, troubleshooting, and FAQ.

---

## 🔑 Key Concepts

### The bevy_eventwork Ecosystem

**bevy_eventwork** is a modular networking ecosystem for Bevy applications:

- **eventwork** - Core networking library (TCP, WebSocket, custom transports)
- **eventwork_sync** - Server-side ECS component synchronization
- **eventwork_client** - Reactive Leptos client library for web UIs
- **eventwork_websockets** - WebSocket transport provider
- **eventwork_memory** - In-memory transport for testing

### Core Features

**Automatic Message Registration** (eventwork)
- Zero boilerplate networking
- Just derive `Serialize + Deserialize`
- Type-safe message handling

**Reflection-Driven Sync** (eventwork_sync)
- Automatic component synchronization
- Opt-in per component type
- Configurable sync settings
- Mutation authorization

**Reactive Subscriptions** (eventwork_client)
- Automatic subscription management
- Fine-grained reactivity with Leptos
- Focus-retaining editable fields
- Built-in DevTools

---

## 📖 Documentation Status

This documentation is currently being built. Check back soon for comprehensive guides!

**Last Updated**: 2025-11-22  
**bevy_eventwork Version**: 1.1.1  
**Bevy Version**: 0.17.2
