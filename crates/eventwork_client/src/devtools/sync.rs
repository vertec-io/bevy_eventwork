//! DevtoolsSync - Leptos-reactive wrapper around SyncClient
//!
//! This module provides a reactive wrapper around eventwork_sync's SyncClient
//! for use in DevTools and other Leptos applications that need direct access
//! to the sync protocol.

use leptos::prelude::*;
use reactive_graph::traits::{Get, Update};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

use eventwork_sync::{
    MutationResponse,
    SerializableEntity,
    SyncClientMessage,
    SyncServerMessage,
    client_registry::ComponentTypeRegistry,
    client_sync::SyncClient,
};

// Re-export MutationState from eventwork_sync for convenience
pub use eventwork_sync::client_sync::MutationState;

/// Leptos-reactive wrapper around `SyncClient` for use in Leptos applications.
///
/// This provides the same API as `SyncClient` but with reactive signals for
/// tracking mutation state in Leptos components.
#[derive(Clone)]
pub struct DevtoolsSync {
    client: Arc<SyncClient>,
    mutations: RwSignal<HashMap<u64, MutationState>>,
}

/// General-purpose sync hook for wiring the eventwork_sync wire protocol
/// into an arbitrary transport (typically a WebSocket using eventwork's
/// binary codec).
///
/// The `send` closure is responsible for serializing and transmitting
/// `SyncClientMessage` values. This keeps the devtools crate agnostic of
/// any particular WebSocket or HTTP client implementation.
///
/// The `registry` is used to serialize mutations from JSON back to the
/// concrete component types expected by the server.
pub fn use_sync(
    send: impl Fn(SyncClientMessage) + Send + Sync + 'static,
    registry: ComponentTypeRegistry,
) -> DevtoolsSync {
    let client = Arc::new(SyncClient::new(send, registry));
    let mutations = RwSignal::new(HashMap::new());

    DevtoolsSync {
        client,
        mutations,
    }
}

impl DevtoolsSync {
    /// Send a raw `SyncClientMessage` without any local bookkeeping.
    ///
    /// This is useful for subscription management or other operations
    /// that don't need per-request client-side tracking.
    pub fn send_raw(&self, message: SyncClientMessage) {
        self.client.send_raw(message);
    }

    /// Read-only view of all tracked mutations keyed by `request_id`.
    pub fn mutations(&self) -> RwSignal<HashMap<u64, MutationState>> {
        self.mutations
    }

    /// Convenience accessor for a single mutation state, if known.
    pub fn mutation_state(&self, request_id: u64) -> Option<MutationState> {
        self.mutations.get().get(&request_id).cloned()
    }

    /// Queue a new mutation for `(entity, component_type)` with the
    /// provided JSON value. Returns the generated `request_id` that will
    /// be echoed back by the server in its `MutationResponse`.
    pub fn mutate(
        &self,
        entity: SerializableEntity,
        component_type: impl Into<String>,
        value: JsonValue,
    ) -> u64 {
        // Delegate to SyncClient
        let request_id = self.client.mutate(entity, component_type, value);

        // Track in reactive signal for Leptos
        self.mutations.update(|map| {
            map.insert(request_id, MutationState::new_pending(request_id));
        });

        request_id
    }

    /// Handle a server-side message, updating mutation state when a
    /// `MutationResponse` is observed.
    pub fn handle_server_message(&self, message: &SyncServerMessage) {
        // Delegate to SyncClient
        self.client.handle_server_message(message);

        // Sync the mutation state to our reactive signal
        self.sync_mutations_from_client();
    }

    /// Helper to handle a `MutationResponse` directly, for cases where
    /// the transport layer already demultiplexes server messages.
    pub fn handle_mutation_response(&self, response: &MutationResponse) {
        // Delegate to SyncClient
        self.client.handle_mutation_response(response);

        // Sync the mutation state to our reactive signal
        self.sync_mutations_from_client();
    }

    /// Sync mutation state from the underlying SyncClient to the reactive signal.
    fn sync_mutations_from_client(&self) {
        let client_mutations = self.client.mutations();
        self.mutations.set(client_mutations);
    }

    /// Get a reference to the underlying SyncClient.
    pub fn client(&self) -> &SyncClient {
        &self.client
    }
}

