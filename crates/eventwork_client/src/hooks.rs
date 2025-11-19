use std::collections::HashMap;

use leptos::prelude::*;

use crate::context::{SyncConnection, SyncContext};
use crate::traits::SyncComponent;

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

