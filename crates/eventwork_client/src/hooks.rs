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

/// Hook for editable component fields with automatic local state management.
///
/// This hook simplifies the pattern of creating editable fields that sync with the server.
/// It manages:
/// - Local state for immediate UI updates
/// - Focus tracking to prevent server updates from overwriting user input
/// - Automatic sync from server to local when not editing
/// - Commit function to send mutations to server
///
/// # Returns
///
/// A tuple of:
/// - `ReadSignal<Option<T>>`: Server value (read-only, from subscription)
/// - `RwSignal<T>`: Local value (read-write, for immediate UI updates)
/// - `WriteSignal<()>`: Commit function (call to send mutation to server)
///
/// # Panics
///
/// Panics if called outside of a `SyncProvider` context.
///
/// # Example
///
/// ```rust,ignore
/// use eventwork_client::{use_sync_component_write, SyncComponent};
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
/// fn PositionEditor(entity_id: u64) -> impl IntoView {
///     let (server_pos, local_pos, commit_pos) =
///         use_sync_component_write::<Position>(entity_id);
///
///     view! {
///         <div>
///             <input
///                 type="number"
///                 prop:value=move || local_pos.get().x.to_string()
///                 on:input=move |ev| {
///                     local_pos.update(|pos| {
///                         pos.x = event_target_value(&ev).parse().unwrap_or(pos.x);
///                     });
///                 }
///                 on:blur=move |_| commit_pos.set(())
///             />
///             <input
///                 type="number"
///                 prop:value=move || local_pos.get().y.to_string()
///                 on:input=move |ev| {
///                     local_pos.update(|pos| {
///                         pos.y = event_target_value(&ev).parse().unwrap_or(pos.y);
///                     });
///                 }
///                 on:blur=move |_| commit_pos.set(())
///             />
///         </div>
///     }
/// }
/// ```
pub fn use_sync_component_write<T: SyncComponent + Clone + Default + 'static>(
    entity_id: u64,
) -> (
    Signal<Option<T>>,
    RwSignal<T>,
    WriteSignal<()>,
) {
    let ctx = expect_context::<SyncContext>();

    // Subscribe to all instances of this component type
    let all_components = use_sync_component::<T>();

    // Create a derived signal for just this entity's component
    let server_value = Signal::derive(move || {
        all_components.get().get(&entity_id).cloned()
    });

    // Create local editable value (initialized with default)
    let local_value = RwSignal::new(T::default());

    // Track if currently editing (has focus)
    let (is_editing, set_is_editing) = signal(false);

    // Sync server -> local when not editing
    Effect::new(move |_| {
        if !is_editing.get() {
            if let Some(value) = server_value.get() {
                local_value.set(value);
            }
        }
    });

    // Create commit signal
    let (commit_trigger, set_commit_trigger) = signal(());

    // Track if this is the first run to avoid sending mutation on mount
    let is_first_run = RwSignal::new(true);

    // Handle commits - use untracked to avoid reactivity to local_value changes
    Effect::new(move |_| {
        commit_trigger.track();

        // Skip the first run to avoid sending mutation on mount
        if is_first_run.get_untracked() {
            is_first_run.set(false);
            return;
        }

        // Read local_value without tracking to avoid infinite loops
        let value = local_value.get_untracked();
        ctx.mutate(entity_id, value);

        set_is_editing.set(false);
    });

    // Return read-only server, read-write local, and commit trigger
    (server_value, local_value, set_commit_trigger)
}

/// Hook for creating controlled input fields with focus tracking.
///
/// This is a lower-level hook that manages the boilerplate of creating an editable
/// field that syncs with a server value but doesn't overwrite user input during editing.
///
/// Unlike `use_sync_component_write`, this hook works with any value type and doesn't
/// automatically send mutations - you provide the commit callback.
///
/// The key feature is that it returns a **derived signal** that switches between the
/// local (editing) value and the server value based on focus state. This prevents
/// the DOM from being updated while the user is editing, which would steal focus.
///
/// # Returns
///
/// A tuple of:
/// - `Signal<T>`: Display value (derived signal that switches between local and server)
/// - `RwSignal<T>`: Local value (write-only, for `on:input` handler)
/// - `WriteSignal<bool>`: Focus setter (call with `true` on focus, `false` on blur)
///
/// # Example
///
/// ```rust,ignore
/// use eventwork_client::use_controlled_input;
///
/// #[component]
/// fn NumberInput(server_value: Signal<String>, on_commit: impl Fn(String) + 'static) -> impl IntoView {
///     let (display_value, local_value, set_is_focused) = use_controlled_input(server_value);
///
///     view! {
///         <input
///             prop:value=move || display_value.get()
///             on:input=move |ev| local_value.set(event_target_value(&ev))
///             on:focus=move |_| set_is_focused.set(true)
///             on:blur=move |_| {
///                 set_is_focused.set(false);
///                 on_commit(local_value.get_untracked());
///             }
///         />
///     }
/// }
/// ```
pub fn use_controlled_input<T: Clone + Send + Sync + 'static>(
    server_value: Signal<T>,
) -> (Signal<T>, RwSignal<T>, WriteSignal<bool>) {
    // Create local editable value
    let local_value = RwSignal::new(server_value.get_untracked());

    // Track if currently focused
    let (is_focused, set_is_focused) = signal(false);

    // Create a derived signal that chooses between local and server value based on focus
    // This prevents DOM updates while editing
    let display_value = Signal::derive(move || {
        if is_focused.get() {
            // While focused, show local value (user's edits)
            local_value.get()
        } else {
            // When not focused, show server value and sync local
            let server = server_value.get();
            local_value.update_untracked(|v| *v = server.clone());
            server
        }
    });

    (display_value, local_value, set_is_focused)
}

