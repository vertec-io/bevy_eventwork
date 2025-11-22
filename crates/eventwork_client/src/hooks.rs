use std::collections::HashMap;

use leptos::prelude::*;

use crate::context::{MutationState, SyncConnection, SyncContext};
use crate::traits::SyncComponent;

#[cfg(feature = "stores")]
use reactive_stores::Store;

/// Hook to subscribe to a component type.
///
/// This returns a signal containing a HashMap of entity_id -> component.
/// The subscription is automatically managed - it will be created when the
/// component mounts and cleaned up when it unmounts.
///
/// Multiple calls to this hook with the same component type will share the
/// same underlying subscription (deduplication).
///
/// # Panics
///
/// Panics if called outside of a `SyncProvider` context.
///
/// # Example
///
/// ```rust,ignore
/// use eventwork_client::{use_sync_component, SyncComponent};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Clone, Default, Serialize, Deserialize)]
/// struct Position {
///     x: f32,
///     y: f32,
/// }
///
/// impl_sync_component!(Position);
///
/// #[component]
/// fn PositionList() -> impl IntoView {
///     let positions = use_sync_component::<Position>();
///
///     view! {
///         <ul>
///             <For
///                 each=move || positions.get().into_iter()
///                 key=|(id, _)| *id
///                 children=|(id, pos)| {
///                     view! {
///                         <li>{format!("Entity {}: ({}, {})", id, pos.x, pos.y)}</li>
///                     }
///                 }
///             />
///         </ul>
///     }
/// }
/// ```
pub fn use_sync_component<T: SyncComponent + Clone + Default + 'static>() -> ReadSignal<HashMap<u64, T>> {
    let ctx = expect_context::<SyncContext>();
    ctx.subscribe_component::<T>()
}

/// Hook to access the WebSocket connection control interface.
///
/// This allows you to manually control the WebSocket connection (open/close)
/// and check the connection state.
///
/// # Panics
///
/// Panics if called outside of a `SyncProvider` context.
///
/// # Example
///
/// ```rust,ignore
/// use eventwork_client::use_sync_connection;
/// use leptos_use::core::ConnectionReadyState;
///
/// #[component]
/// fn ConnectionStatus() -> impl IntoView {
///     let connection = use_sync_connection();
///
///     let status_text = move || {
///         match connection.ready_state.get() {
///             ConnectionReadyState::Connecting => "Connecting...",
///             ConnectionReadyState::Open => "Connected",
///             ConnectionReadyState::Closing => "Closing...",
///             ConnectionReadyState::Closed => "Disconnected",
///         }
///     };
///
///     let is_connected = move || {
///         connection.ready_state.get() == ConnectionReadyState::Open
///     };
///
///     view! {
///         <div>
///             <p>"Status: " {status_text}</p>
///             <button
///                 on:click=move |_| (connection.open)()
///                 disabled=is_connected
///             >
///                 "Connect"
///             </button>
///             <button
///                 on:click=move |_| (connection.close)()
///                 disabled=move || !is_connected()
///             >
///                 "Disconnect"
///             </button>
///         </div>
///     }
/// }
/// ```
pub fn use_sync_connection() -> SyncConnection {
    let ctx = expect_context::<SyncContext>();
    ctx.connection()
}

/// Hook to subscribe to a component type with fine-grained reactivity using stores.
///
/// This returns a `Store<HashMap<u64, T>>` that provides fine-grained reactive access
/// to individual entity fields. Unlike signals which are atomic, stores allow you to
/// reactively access nested fields without triggering updates for unrelated data.
///
/// The subscription is automatically managed - it will be created when the component
/// mounts and cleaned up when it unmounts.
///
/// Multiple calls to this hook with the same component type will share the same
/// underlying subscription (deduplication).
///
/// # Panics
///
/// Panics if called outside of a `SyncProvider` context.
///
/// # Example
///
/// ```rust,ignore
/// use eventwork_client::{use_sync_component_store, SyncComponent};
/// use reactive_stores::Store;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Clone, Default, Serialize, Deserialize, Store)]
/// struct Position {
///     x: f32,
///     y: f32,
/// }
///
/// impl_sync_component!(Position);
///
/// #[component]
/// fn PositionList() -> impl IntoView {
///     let positions = use_sync_component_store::<Position>();
///
///     view! {
///         <For
///             each=move || positions.read().keys().copied().collect::<Vec<_>>()
///             key=|id| *id
///             let:entity_id
///         >
///             {move || {
///                 // Fine-grained: only updates when this specific entity's position changes
///                 let pos = positions.read().get(&entity_id).cloned();
///                 view! {
///                     <li>{format!("Entity {}: {:?}", entity_id, pos)}</li>
///                 }
///             }}
///         </For>
///     }
/// }
/// ```
#[cfg(feature = "stores")]
pub fn use_sync_component_store<T: SyncComponent + Clone + Default + 'static>() -> Store<HashMap<u64, T>> {
    let ctx = expect_context::<SyncContext>();
    ctx.subscribe_component_store::<T>()
}

/// Hook to access the SyncContext directly.
///
/// This provides access to the full SyncContext API, including mutation methods.
/// Most users should use the more specific hooks like `use_sync_component` or
/// `use_sync_mutations` instead.
///
/// # Panics
///
/// Panics if called outside of a `SyncProvider` context.
///
/// # Example
///
/// ```rust,ignore
/// use eventwork_client::use_sync_context;
///
/// #[component]
/// fn MutatePosition() -> impl IntoView {
///     let ctx = use_sync_context();
///
///     let update_position = move |_| {
///         let new_pos = Position { x: 10.0, y: 20.0 };
///         let request_id = ctx.mutate(entity_id, new_pos);
///     };
///
///     view! {
///         <button on:click=update_position>"Update Position"</button>
///     }
/// }
/// ```
pub fn use_sync_context() -> SyncContext {
    expect_context::<SyncContext>()
}

/// Hook to access mutation state tracking.
///
/// This returns a read-only signal containing all mutation states, allowing
/// components to reactively track the status of mutations (pending, success, error).
///
/// # Panics
///
/// Panics if called outside of a `SyncProvider` context.
///
/// # Example
///
/// ```rust,ignore
/// use eventwork_client::{use_sync_context, use_sync_mutations};
/// use eventwork_sync::MutationStatus;
///
/// #[component]
/// fn MutateWithFeedback() -> impl IntoView {
///     let ctx = use_sync_context();
///     let mutations = use_sync_mutations();
///     let (last_request_id, set_last_request_id) = signal(None::<u64>);
///
///     let update_position = move |_| {
///         let new_pos = Position { x: 10.0, y: 20.0 };
///         let request_id = ctx.mutate(entity_id, new_pos);
///         set_last_request_id.set(Some(request_id));
///     };
///
///     let status_text = move || {
///         last_request_id.get().and_then(|id| {
///             mutations.get().get(&id).map(|state| {
///                 match &state.status {
///                     None => "Pending...".to_string(),
///                     Some(MutationStatus::Ok) => "Success!".to_string(),
///                     Some(status) => format!("Error: {:?}", status),
///                 }
///             })
///         })
///     };
///
///     view! {
///         <div>
///             <button on:click=update_position>"Update Position"</button>
///             {move || status_text().unwrap_or_default()}
///         </div>
///     }
/// }
/// ```
pub fn use_sync_mutations() -> ReadSignal<HashMap<u64, MutationState>> {
    let ctx = expect_context::<SyncContext>();
    ctx.mutations()
}

