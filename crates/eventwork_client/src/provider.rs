use std::sync::Arc;

use leptos::prelude::*;
use leptos_use::{use_websocket_with_options, DummyEncoder, UseWebSocketOptions, UseWebSocketReturn};
use eventwork_common::codec::EventworkBincodeCodec;
use eventwork_common::NetworkPacket;

use crate::context::SyncContext;
use crate::error::SyncError;
use crate::registry::ClientRegistry;
use eventwork_sync::{SyncClientMessage, SyncServerMessage};

/// Provider component that sets up WebSocket connection and provides SyncContext.
///
/// This component should wrap your application or the part of your application
/// that needs access to synchronized ECS data.
///
/// # Example
///
/// ```rust,ignore
/// use eventwork_client::{SyncProvider, ClientRegistryBuilder};
///
/// #[component]
/// pub fn App() -> impl IntoView {
///     let registry = ClientRegistryBuilder::new()
///         .register::<Position>()
///         .register::<Velocity>()
///         .build();
///
///     view! {
///         <SyncProvider
///             url="ws://localhost:3000/sync"
///             registry=registry
///         >
///             <MyGameUI />
///         </SyncProvider>
///     }
/// }
/// ```
#[component]
pub fn SyncProvider(
    /// WebSocket URL to connect to
    url: String,
    /// Type registry for deserializing components
    registry: Arc<ClientRegistry>,
    /// Whether to automatically connect on mount (default: true)
    #[prop(optional)]
    auto_connect: Option<bool>,
    /// Child components
    children: Children,
) -> impl IntoView {
    let auto_connect = auto_connect.unwrap_or(true);

    // Set up WebSocket connection using NetworkPacket wrapper
    // This matches the eventwork wire protocol
    let UseWebSocketReturn {
        ready_state,
        message: raw_message,
        send: raw_send,
        open,
        close,
        ..
    } = use_websocket_with_options::<
        NetworkPacket,
        NetworkPacket,
        EventworkBincodeCodec,
        (),
        DummyEncoder,
    >(
        &url,
        UseWebSocketOptions::default(),
    );

    // Auto-connect if requested
    if auto_connect {
        open();
    }

    // Create error signal
    let last_error = RwSignal::new(None::<SyncError>);

    // Create SyncContext
    // Wrap the send function to convert SyncClientMessage to NetworkPacket
    let send_arc = Arc::new(move |data: &[u8]| {
        // Deserialize the bytes to SyncClientMessage
        if let Ok((msg, _)) = bincode::serde::decode_from_slice::<SyncClientMessage, _>(
            data,
            bincode::config::standard(),
        ) {
            // Wrap in NetworkPacket and send
            let packet = NetworkPacket {
                type_name: std::any::type_name::<SyncClientMessage>().to_string(),
                schema_hash: 0, // Schema hash not used for sync messages
                data: bincode::serde::encode_to_vec(&msg, bincode::config::standard()).unwrap(),
            };
            raw_send(&packet);
        }
    });

    let open_arc = Arc::new(move || {
        open();
    });

    let close_arc = Arc::new(move || {
        close();
    });

    let ctx = SyncContext::new(
        ready_state.into(),
        last_error.into(),
        send_arc,
        open_arc,
        close_arc,
        registry.clone(),
    );

    // Provide context to children
    provide_context(ctx.clone());

    // Set up message handler
    Effect::new(move || {
        if let Some(packet) = raw_message.get() {
            // Unwrap NetworkPacket and deserialize to SyncServerMessage
            match bincode::serde::decode_from_slice::<SyncServerMessage, _>(
                &packet.data,
                bincode::config::standard(),
            ) {
                Ok((server_msg, _)) => {
                    handle_server_message(&ctx, server_msg, &last_error);
                }
                Err(e) => {
                    last_error.set(Some(SyncError::DeserializationFailed {
                        component_name: "SyncServerMessage".to_string(),
                        error: format!("Failed to deserialize from NetworkPacket: {}", e),
                    }));
                }
            }
        }
    });

    // Render children
    children()
}

/// Handle incoming server messages.
fn handle_server_message(
    ctx: &SyncContext,
    msg: SyncServerMessage,
    last_error: &RwSignal<Option<SyncError>>,
) {
    match msg {
        SyncServerMessage::SyncBatch(batch) => {
            // Process each sync item in the batch
            for item in batch.items {
                if let Err(e) = handle_sync_item(ctx, item) {
                    last_error.set(Some(e));
                }
            }
        }
        SyncServerMessage::MutationResponse(_response) => {
            // TODO: Handle mutation responses when we implement mutations
        }
        SyncServerMessage::QueryResponse(_response) => {
            // TODO: Handle query responses when we implement queries
        }
    }
}

/// Handle a single sync item.
fn handle_sync_item(
    _ctx: &SyncContext,
    item: eventwork_sync::SyncItem,
) -> Result<(), SyncError> {
    use eventwork_sync::SyncItem;

    #[cfg(target_arch = "wasm32")]
    let is_snapshot = matches!(&item, SyncItem::Snapshot { .. });

    match item {
        SyncItem::Snapshot {
            subscription_id: _,
            entity,
            component_type,
            value,
        } | SyncItem::Update {
            subscription_id: _,
            entity,
            component_type,
            value,
        } => {
            let entity_id = entity.bits;

            // Log for debugging
            #[cfg(target_arch = "wasm32")]
            {
                leptos::logging::log!(
                    "[SyncProvider] Received {} for entity {} component {} ({} bytes)",
                    if is_snapshot { "Snapshot" } else { "Update" },
                    entity_id,
                    component_type,
                    value.len()
                );
            }

            // TODO: Update the signal for this component type
            // This requires storing type-erased update functions in the signal cache
            // or using a different architecture (e.g., storing signals by component name)
            // For now, we just log the message - the DevTools will show the data

            Ok(())
        }
        SyncItem::ComponentRemoved {
            subscription_id: _,
            entity,
            component_type,
        } => {
            let entity_id = entity.bits;

            #[cfg(target_arch = "wasm32")]
            {
                leptos::logging::log!(
                    "[SyncProvider] Component {} removed from entity {}",
                    component_type,
                    entity_id
                );
            }

            // TODO: Phase 2 - Remove component from signal
            Ok(())
        }
        SyncItem::EntityRemoved {
            subscription_id: _,
            entity,
        } => {
            let entity_id = entity.bits;

            #[cfg(target_arch = "wasm32")]
            {
                leptos::logging::log!(
                    "[SyncProvider] Entity {} removed",
                    entity_id
                );
            }

            // TODO: Phase 2 - Handle entity removal across all component types
            Ok(())
        }
    }
}

