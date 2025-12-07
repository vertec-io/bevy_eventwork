# Cleanup Plan - bevy_eventwork

## 1. Deprecated Code to Remove ✅ COMPLETE

### eventwork_client crate
- [x] `ClientRegistry` struct (registry.rs) - REMOVED (entire file deleted)
- [x] `ClientRegistryBuilder` struct (registry.rs) - REMOVED (entire file deleted)
- [x] Old `impl_sync_component!` macro (traits.rs) - REMOVED
- [x] Updated lib.rs to remove deprecated exports
- [x] Updated documentation examples to use `ClientTypeRegistry`

### eventwork_sync crate
- [x] `ComponentTypeRegistry` struct (client_registry.rs) - REMOVED (file deleted)
- [x] `client_registry` module (lib.rs) - REMOVED
- [x] `client_sync` module (lib.rs) - REMOVED
- [x] `client_sync.rs` file - REMOVED (file deleted)

### eventwork_client devtools
- [x] Updated devtools/mod.rs to use local `MutationState` instead of deprecated import
- [x] Updated documentation examples to use `ClientTypeRegistry::builder()`

### PRESERVE (as requested)
- [x] `eventwork_client::native_client` module - KEPT (deprecated but preserved)

## 2. Examples Consolidation ✅ COMPLETE

### Decision: Keep Low-Level Examples in Crates

After analysis, we decided to **KEEP** the low-level examples in their respective crates:

#### crates/eventwork/examples - KEEP
- `client.rs` - TCP client example (shows base eventwork API)
- `server.rs` - TCP server example
- `automatic_messages.rs` - Automatic message registration pattern
- `shared.rs` - Shared types
- **Rationale**: These demonstrate the low-level eventwork API without the sync layer

#### crates/eventwork_websockets/examples - KEEP
- `client.rs` - WebSocket client example
- `server.rs` - WebSocket server example
- `hybrid_server.rs` - Hybrid TCP/WebSocket server (advanced pattern)
- `immediate_messages.rs` - Immediate message pattern (architectural choice)
- `scheduled_messages.rs` - Scheduled message pattern (architectural choice)
- `shared.rs` / `shared_types.rs` - Shared types
- **Rationale**: These demonstrate advanced WebSocket patterns and architectural choices

### crates/eventwork_sync/examples - REMOVED ✅
- [x] Entire directory removed - all examples were duplicates with old import paths
- [x] `devtools-demo-server.rs` - Duplicate of examples/devtools-demo/server
- [x] `fanuc_server.rs` - Duplicate of examples/fanuc/server (simple simulator)
- [x] `fanuc_real_server.rs` - Duplicate of examples/fanuc/server (real FANUC RMI)

### crates/eventwork_client/examples - REMOVED ✅
- [x] Entire directory removed earlier
- [x] `basic_server.rs` - Duplicate of examples/basic/server
- [x] `basic_client/` - Duplicate of examples/basic/client
- [x] `fanuc_real_client/` - Duplicate of examples/fanuc/client

## 3. Final Examples Structure ✅

```
examples/                           # High-level sync examples
├── basic/                          # eventwork_sync + eventwork_client example
│   ├── client/                    # Leptos WASM client
│   └── server/                    # Bevy server with sync
├── devtools-demo/                 # DevTools demonstration
│   └── server/
├── fanuc/                         # FANUC robot simulation
│   ├── client/
│   └── server/
└── shared/                        # Shared types for high-level examples
    ├── basic_types/
    ├── demo_types/
    ├── fanuc_types/
    └── fanuc_real_types/

crates/eventwork/examples/         # Low-level TCP examples (KEPT)
├── client.rs
├── server.rs
├── automatic_messages.rs
└── shared.rs

crates/eventwork_websockets/examples/  # Advanced WebSocket patterns (KEPT)
├── client.rs
├── server.rs
├── hybrid_server.rs
├── immediate_messages.rs
├── scheduled_messages.rs
├── shared.rs
└── shared_types.rs
```

## 4. Summary of Changes ✅

### ✅ Phase 1: Remove Deprecated Code - COMPLETE
1. ✅ Removed `crates/eventwork_client/src/registry.rs` (entire file)
2. ✅ Removed `ClientRegistry` and `ClientRegistryBuilder` exports from lib.rs
3. ✅ Removed `impl_sync_component!` macro from traits.rs
4. ✅ Removed `crates/eventwork_sync/src/client_registry.rs` (entire file)
5. ✅ Removed `crates/eventwork_sync/src/client_sync.rs` (entire file)
6. ✅ Removed deprecated module declarations from eventwork_sync/lib.rs
7. ✅ Updated devtools/mod.rs to use local MutationState
8. ✅ Updated all documentation examples to use `ClientTypeRegistry::builder()`
9. ✅ Verified compilation - all workspace crates compile successfully

### ✅ Phase 2: Consolidate Examples - COMPLETE
1. ✅ Removed `crates/eventwork_sync/examples/` (entire directory - all duplicates)
2. ✅ Removed `crates/eventwork_client/examples/` (done earlier)
3. ✅ Kept `crates/eventwork/examples/` - demonstrates low-level TCP API
4. ✅ Kept `crates/eventwork_websockets/examples/` - demonstrates advanced patterns
5. ✅ Verified compilation - all examples compile

### ✅ Phase 3: Remove Old Shared Crates - COMPLETE
1. ✅ Removed `crates/demo_shared/` (entire directory)
2. ✅ Removed `crates/eventwork_client_example_shared/` (entire directory)
3. ✅ Removed `crates/fanuc_real_shared/` (entire directory)
4. ✅ Removed `crates/fanuc_shared/` (entire directory)
5. ✅ Removed references from `crates/eventwork_sync/Cargo.toml` dev-dependencies
6. ✅ Removed references from `crates/eventwork_client/Cargo.toml` dev-dependencies
7. ✅ Removed example declarations from `crates/eventwork_sync/Cargo.toml`

### ✅ Phase 4: Final Verification - COMPLETE
1. ✅ Workspace compiles with only expected warnings (unused variables in examples)
2. ✅ No deprecated code remains (except preserved `native_client` module)
3. ✅ No duplicate examples remain
4. ✅ No old shared crates remain in `/crates` directory
5. ✅ Documentation updated to use new API

