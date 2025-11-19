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

// Module declarations
mod context;
mod error;
mod hooks;
mod provider;
mod registry;
mod traits;

// Re-exports
pub use context::{SyncConnection, SyncContext};
pub use error::SyncError;
pub use hooks::{use_sync_component, use_sync_connection};
pub use provider::SyncProvider;
pub use registry::{ClientRegistry, ClientRegistryBuilder};
pub use traits::SyncComponent;

// Conditional re-exports
// #[cfg(feature = "stores")]
// pub use hooks::use_sync_component_store;

// #[cfg(feature = "devtools")]
// pub mod devtools;

