//! # Eventwork Client
//!
//! High-level reactive client library for `eventwork_sync` with Leptos integration.
//!
//! This library provides ergonomic hooks and components for building reactive web applications
//! that synchronize with Bevy ECS servers via `eventwork_sync`.
//!
//! ## Features
//!
//! - **Automatic Subscription Management**: Subscribe to components with a single hook call
//! - **Subscription Deduplication**: Multiple components share subscriptions automatically
//! - **Lifecycle Management**: Auto-subscribe on mount, auto-unsubscribe on unmount
//! - **Reconnection Handling**: Automatic re-subscription on reconnect
//! - **Type Safety**: Compile-time type checking with Rust's type system
//! - **Dual API**: Support for both signals (atomic) and stores (fine-grained reactivity)
//!
//! ## Quick Start
//!
//! ### Read-Only Display
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use eventwork_client::{SyncProvider, use_sync_component, ClientRegistryBuilder};
//!
//! #[component]
//! fn App() -> impl IntoView {
//!     let registry = ClientRegistryBuilder::new()
//!         .register::<Position>()
//!         .register::<Velocity>()
//!         .build();
//!
//!     view! {
//!         <SyncProvider url="ws://localhost:8080" registry=registry>
//!             <GameView/>
//!         </SyncProvider>
//!     }
//! }
//!
//! #[component]
//! fn GameView() -> impl IntoView {
//!     // Automatically subscribes, updates, and unsubscribes
//!     let positions = use_sync_component::<Position>();
//!
//!     view! {
//!         <For
//!             each=move || positions.get().iter().map(|(id, pos)| (*id, pos.clone())).collect::<Vec<_>>()
//!             key=|(id, _)| *id
//!             let:item
//!         >
//!             {
//!                 let (entity_id, position) = item;
//!                 view! {
//!                     <div>"Entity " {entity_id} ": " {format!("{:?}", position)}</div>
//!                 }
//!             }
//!         </For>
//!     }
//! }
//! ```
//!
//! ### Editable Fields with Focus Retention
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use eventwork_client::{SyncFieldInput, SyncComponent};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Clone, Default, Serialize, Deserialize)]
//! struct Position {
//!     x: f32,
//!     y: f32,
//! }
//!
//! // SyncComponent is automatically implemented!
//!
//! #[component]
//! fn PositionEditor(entity_id: u64) -> impl IntoView {
//!     view! {
//!         <div class="position-editor">
//!             <label>
//!                 "X: "
//!                 <SyncFieldInput
//!                     entity_id=entity_id
//!                     field_accessor=|pos: &Position| pos.x
//!                     field_mutator=|pos: &Position, new_x: f32| Position { x: new_x, y: pos.y }
//!                     input_type="number"
//!                     class="number-input"
//!                 />
//!             </label>
//!             <label>
//!                 "Y: "
//!                 <SyncFieldInput
//!                     entity_id=entity_id
//!                     field_accessor=|pos: &Position| pos.y
//!                     field_mutator=|pos: &Position, new_y: f32| Position { x: pos.x, y: new_y }
//!                     input_type="number"
//!                     class="number-input"
//!                 />
//!             </label>
//!         </div>
//!     }
//! }
//! ```
//!
//! The `SyncFieldInput` component implements:
//! - ✅ Focus retention through server updates
//! - ✅ User input preservation while focused
//! - ✅ Enter key to apply mutation
//! - ✅ Blur (click away) to revert to server value

// Module declarations
mod components;
mod context;
mod error;
mod hooks;
mod provider;
mod registry;
mod traits;

// Re-exports
pub use components::SyncFieldInput;
pub use context::{MutationState, SyncConnection, SyncContext};
pub use error::SyncError;
pub use hooks::{
    use_controlled_input, use_sync_component, use_sync_component_write, use_sync_connection,
    use_sync_context, use_sync_field_editor, use_sync_mutations,
};
pub use provider::SyncProvider;
pub use registry::{ClientRegistry, ClientRegistryBuilder};
pub use traits::SyncComponent;

// Re-export mutation types from eventwork_sync for convenience
pub use eventwork_sync::MutationStatus;

// Conditional re-exports
#[cfg(feature = "stores")]
pub use hooks::use_sync_component_store;

#[cfg(feature = "devtools")]
pub mod devtools;

#[cfg(all(feature = "devtools", target_arch = "wasm32"))]
pub use devtools::{DevTools, DevToolsMode};

