//! Client-side synchronization utilities for eventwork_sync.
//!
//! This module provides client-side utilities for interacting with an eventwork_sync server.
//! It is NOT Bevy-specific and can be used from any Rust client (WASM, native, etc.).
//!
//! The main type is [`SyncClient`], which provides:
//! - Sending mutations to the server
//! - Tracking mutation responses
//! - Sending subscription requests
//! - Handling server messages
//!
//! This module is available when the `client` feature is enabled (which is enabled by default).

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value as JsonValue;

use crate::{
    MutateComponent,
    MutationResponse,
    MutationStatus,
    SerializableEntity,
    SyncClientMessage,
    SyncServerMessage,
    client_registry::ComponentTypeRegistry,
};

/// Per-request mutation state tracked on the client.
#[derive(Clone, Debug)]
pub struct MutationState {
    pub request_id: u64,
    pub status: Option<MutationStatus>,
    pub message: Option<String>,
}

impl MutationState {
    pub fn new_pending(request_id: u64) -> Self {
        Self {
            request_id,
            status: None,
            message: None,
        }
    }
}

/// Client-side synchronization manager for eventwork_sync.
///
/// This provides a high-level API for:
/// - Sending mutations to the server
/// - Tracking mutation responses
/// - Sending subscription/unsubscribe requests
/// - Handling server messages
///
/// The `SyncClient` is transport-agnostic - you provide a `send` function
/// that handles the actual transmission of messages (typically over WebSocket).
///
/// # Example
/// ```ignore
/// use eventwork_sync::client_sync::SyncClient;
/// use eventwork_sync::client_registry::ComponentTypeRegistry;
///
/// let mut registry = ComponentTypeRegistry::new();
/// registry.register::<MyComponent>();
///
/// let client = SyncClient::new(
///     |msg| { /* send msg over websocket */ },
///     registry
/// );
///
/// // Send a mutation
/// let request_id = client.mutate(entity, "MyComponent", json_value);
///
/// // Later, when you receive a server message:
/// client.handle_server_message(&server_msg);
/// ```
pub struct SyncClient {
    send: Arc<dyn Fn(SyncClientMessage) + Send + Sync>,
    next_request_id: std::sync::Mutex<u64>,
    mutations: std::sync::Mutex<HashMap<u64, MutationState>>,
    registry: ComponentTypeRegistry,
}

impl SyncClient {
    /// Create a new `SyncClient` with the given send function and type registry.
    ///
    /// The `send` function is responsible for serializing and transmitting
    /// `SyncClientMessage` values over the wire (typically via WebSocket).
    pub fn new(
        send: impl Fn(SyncClientMessage) + Send + Sync + 'static,
        registry: ComponentTypeRegistry,
    ) -> Self {
        Self {
            send: Arc::new(send),
            next_request_id: std::sync::Mutex::new(0),
            mutations: std::sync::Mutex::new(HashMap::new()),
            registry,
        }
    }

    /// Send a raw `SyncClientMessage` without any local bookkeeping.
    ///
    /// This is useful for subscription management or other operations
    /// that don't need per-request client-side tracking.
    pub fn send_raw(&self, message: SyncClientMessage) {
        (self.send)(message);
    }

    /// Get a snapshot of all tracked mutations.
    pub fn mutations(&self) -> HashMap<u64, MutationState> {
        self.mutations.lock().unwrap().clone()
    }

    /// Get the state of a specific mutation by request_id.
    pub fn mutation_state(&self, request_id: u64) -> Option<MutationState> {
        self.mutations.lock().unwrap().get(&request_id).cloned()
    }

    /// Queue a new mutation for `(entity, component_type)` with the
    /// provided JSON value. Returns the generated `request_id` that will
    /// be echoed back by the server in its `MutationResponse`.
    ///
    /// # Arguments
    /// - `entity`: The entity to mutate (use `SerializableEntity::DANGLING` to spawn a new entity)
    /// - `component_type`: The component type name (e.g., "RobotPosition")
    /// - `value`: The component value as a JSON object
    ///
    /// # Returns
    /// The request_id that can be used to track the mutation status.
    pub fn mutate(
        &self,
        entity: SerializableEntity,
        component_type: impl Into<String>,
        value: JsonValue,
    ) -> u64 {
        let request_id = {
            let mut next_id = self.next_request_id.lock().unwrap();
            *next_id += 1;
            *next_id
        };

        // Track the pending mutation locally
        self.mutations.lock().unwrap().insert(
            request_id,
            MutationState::new_pending(request_id),
        );

        let component_type_str = component_type.into();

        // Use the type registry to serialize JSON → concrete type → bincode bytes
        let value_bytes = match self.registry.serialize_by_name(&component_type_str, &value) {
            Ok(bytes) => bytes,
            Err(e) => {
                log::error!(
                    "Failed to serialize mutation for '{}': {}",
                    component_type_str, e
                );
                return request_id;
            }
        };

        let msg = SyncClientMessage::Mutate(MutateComponent {
            request_id: Some(request_id),
            entity,
            component_type: component_type_str,
            value: value_bytes,
        });

        (self.send)(msg);

        request_id
    }

    /// Handle a server-side message, updating mutation state when a
    /// `MutationResponse` is observed.
    pub fn handle_server_message(&self, message: &SyncServerMessage) {
        if let SyncServerMessage::MutationResponse(response) = message {
            self.handle_mutation_response(response);
        }
    }

    /// Handle a `MutationResponse` directly, for cases where
    /// the transport layer already demultiplexes server messages.
    pub fn handle_mutation_response(&self, response: &MutationResponse) {
        if let Some(request_id) = response.request_id {
            let mut mutations = self.mutations.lock().unwrap();
            mutations
                .entry(request_id)
                .and_modify(|state| {
                    state.status = Some(response.status.clone());
                    state.message = response.message.clone();
                })
                .or_insert_with(|| MutationState {
                    request_id,
                    status: Some(response.status.clone()),
                    message: response.message.clone(),
                });
        }
    }

    /// Get a reference to the component type registry.
    pub fn registry(&self) -> &ComponentTypeRegistry {
        &self.registry
    }
}

impl Clone for SyncClient {
    fn clone(&self) -> Self {
        Self {
            send: Arc::clone(&self.send),
            next_request_id: std::sync::Mutex::new(*self.next_request_id.lock().unwrap()),
            mutations: std::sync::Mutex::new(self.mutations.lock().unwrap().clone()),
            registry: self.registry.clone(),
        }
    }
}

