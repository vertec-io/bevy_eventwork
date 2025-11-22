//! DevTools module for eventwork_client
//!
//! This module provides a drop-in DevTools widget for debugging and inspecting
//! ECS entities synchronized via eventwork_sync.
//!
//! ## Features
//!
//! - Hierarchical entity inspector
//! - Real-time component editing with mutations
//! - Controlled input pattern (prevents server updates during editing)
//! - Type registry for JSON serialization/deserialization
//!
//! ## Usage
//!
//! Enable the `devtools` feature in your Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! eventwork_client = { version = "0.1", features = ["devtools"] }
//! ```
//!
//! Then use the DevTools component in your Leptos app:
//!
//! ```rust,ignore
//! use eventwork_client::devtools::DevTools;
//! use eventwork_sync::client_registry::ComponentTypeRegistry;
//!
//! let mut registry = ComponentTypeRegistry::new();
//! registry.register::<Position>();
//! registry.register::<Velocity>();
//!
//! view! {
//!     <DevTools url="ws://127.0.0.1:3000/sync" registry=registry />
//! }
//! ```

mod sync;

#[cfg(target_arch = "wasm32")]
mod ui;

// Re-export public API
pub use sync::{DevtoolsSync, use_sync};

#[cfg(target_arch = "wasm32")]
pub use ui::{DevTools, DevToolsMode};

// Re-export MutationState from eventwork_sync for convenience
pub use eventwork_sync::client_sync::MutationState;

// Re-export core wire-level types so downstream tools can depend on this
// crate alone for typical sync workflows.
pub use eventwork_sync::{
    MutateComponent as SyncMutateComponent,
    MutationResponse as SyncMutationResponse,
    MutationStatus as SyncMutationStatus,
    SerializableEntity as SyncSerializableEntity,
    SyncBatch,
    SyncClientMessage as SyncClientMsg,
    SyncItem,
    SyncServerMessage as SyncServerMsg,
    SubscriptionRequest,
    UnsubscribeRequest,
};

