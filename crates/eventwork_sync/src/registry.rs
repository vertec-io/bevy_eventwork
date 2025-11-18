use bevy::prelude::*;
use std::sync::Arc;

use crate::messages::{MutationStatus, SerializableEntity};

/// Configuration for how a component type should be synchronized.
#[derive(Clone)]
pub struct ComponentSyncConfig {
    /// Maximum number of updates per frame (per client); `None` means unlimited.
    pub max_updates_per_frame: Option<usize>,
}

impl Default for ComponentSyncConfig {
    fn default() -> Self {
        Self {
            max_updates_per_frame: None,
        }
    }
}

/// Per-type registration data stored in the [`SyncRegistry`].
pub struct ComponentRegistration {
    pub type_id: std::any::TypeId,
    pub type_name: String,
    pub config: ComponentSyncConfig,
    /// Type-specific function that knows how to deserialize and apply a
    /// [`QueuedMutation`] for this component.
    pub apply_mutation: fn(&mut World, &QueuedMutation) -> MutationStatus,
    /// Type-specific function that can produce a full snapshot of all
    /// `(Entity, Component)` pairs for this component type, encoded as bincode
    /// bytes suitable for transmission over the wire.
    pub snapshot_all: fn(&mut World) -> Vec<(SerializableEntity, Vec<u8>)>,
}

/// Registry of component types that participate in synchronization.
#[derive(Resource, Default)]
pub struct SyncRegistry {
    pub components: Vec<ComponentRegistration>,
}

impl SyncRegistry {
    pub fn register_component(&mut self, registration: ComponentRegistration) {
        // Avoid double registration for the same TypeId.
        if self
            .components
            .iter()
            .any(|c| c.type_id == registration.type_id)
        {
            return;
        }
        self.components.push(registration);
    }
}

/// Subscription tracking keyed by (connection, subscription_id).
#[derive(Resource, Default)]
pub struct SubscriptionManager {
    // For v1, keep this simple; we can optimize later.
    pub subscriptions: Vec<SubscriptionEntry>,
}

/// One subscription from a specific client.
#[derive(Clone)]
pub struct SubscriptionEntry {
    pub connection_id: eventwork_common::ConnectionId,
    pub subscription_id: u64,
    pub component_type: String,
    pub entity: Option<SerializableEntity>,
}

impl SubscriptionManager {
    pub fn add_subscription(&mut self, entry: SubscriptionEntry) {
        self.subscriptions.push(entry);
    }

    pub fn remove_subscription(
        &mut self,
        connection: eventwork_common::ConnectionId,
        subscription_id: u64,
    ) {
        self.subscriptions.retain(|s| {
            !(s.connection_id == connection && s.subscription_id == subscription_id)
        });
    }

    pub fn remove_all_for_connection(&mut self, connection: eventwork_common::ConnectionId) {
        self.subscriptions
            .retain(|s| s.connection_id != connection);
    }
}

/// A single snapshot request queued when a client first subscribes.
#[derive(Clone)]
pub struct SnapshotRequest {
    pub connection_id: eventwork_common::ConnectionId,
    pub subscription_id: u64,
    pub component_type: String,
    pub entity: Option<SerializableEntity>,
}

/// Queue of pending snapshot requests to be processed by a dedicated system.
#[derive(Resource, Default)]
pub struct SnapshotQueue {
    pub pending: Vec<SnapshotRequest>,
}
/// Queue of pending component mutations requested by clients.
#[derive(Resource, Default)]
pub struct MutationQueue {
    /// Pending mutations received from clients via [`SyncClientMessage::Mutate`].
    ///
    /// These are processed by an internal system which will consult any
    /// configured [`MutationAuthorizer`] and, if authorized, deserialize and
    /// apply the change to the ECS world.
    pub pending: Vec<QueuedMutation>,
}

/// A single queued mutation request.
#[derive(Clone)]
pub struct QueuedMutation {
    /// Connection that originated the mutation request.
    pub connection_id: eventwork_common::ConnectionId,
    /// Optional client-chosen correlation id.
    pub request_id: Option<u64>,
    pub entity: SerializableEntity,
    pub component_type: String,
    /// Full component value encoded as bincode bytes (v1 uses full replacement semantics).
    pub value: Vec<u8>,
}

/// Context passed into a [`MutationAuthorizer`] when deciding whether to allow
/// a mutation.
pub struct MutationAuthContext<'a> {
    pub world: &'a World,
}

/// Pluggable policy for deciding whether a queued mutation is allowed to be
/// applied to the world.
///
/// Implementations can inspect arbitrary application state via the
/// [`MutationAuthContext::world`] reference (for example, relationships between
/// connections and entities using Bevy's built-in parent/child hierarchy).
pub trait MutationAuthorizer: Send + Sync + 'static {
    /// Decide whether `mutation` should be applied.
    ///
    /// Returning any status other than [`MutationStatus::Ok`] will prevent the
    /// mutation from being applied and will be propagated back to the client via
    /// [`MutationResponse`].
    fn authorize(&self, ctx: &MutationAuthContext, mutation: &QueuedMutation) -> MutationStatus;
}

/// Resource wrapping the active mutation authorization policy, if any.
///
/// If this resource is not present, all client mutations are treated as
/// authorized by default. Applications can install their own policy by
/// inserting this resource into the `App`.
#[derive(Resource)]
pub struct MutationAuthorizerResource {
    pub inner: Arc<dyn MutationAuthorizer>,
}

impl MutationAuthorizerResource {
    /// Construct an authorizer from a simple closure.
    ///
    /// This is the most convenient way for downstream apps to express custom
    /// authorization logic.
    pub fn from_fn<F>(f: F) -> Self
    where
        F: Fn(&World, &QueuedMutation) -> MutationStatus + Send + Sync + 'static,
    {
        struct ClosureAuthorizer<F>(F);

        impl<F> MutationAuthorizer for ClosureAuthorizer<F>
        where
            F: Fn(&World, &QueuedMutation) -> MutationStatus + Send + Sync + 'static,
        {
            fn authorize(
                &self,
                ctx: &MutationAuthContext,
                mutation: &QueuedMutation,
            ) -> MutationStatus {
                (self.0)(ctx.world, mutation)
            }
        }

        Self {
            inner: Arc::new(ClosureAuthorizer(f)),
        }
    }

    /// Convenience constructor for a built-in "server-only" policy.
    ///
    /// Under this policy, only the special `ConnectionId::SERVER` is allowed to
    /// issue mutations. All other clients will receive
    /// [`MutationStatus::Forbidden`].
    pub fn server_only() -> Self {
        Self {
            inner: Arc::new(ServerOnlyMutationAuthorizer),
        }
    }
}

/// Simple built-in policy that only allows mutations originating from the
/// server connection id. This is useful for deployments where the server is the
/// sole authority that ever mutates ECS state, while clients are strictly
/// read-only observers.
pub struct ServerOnlyMutationAuthorizer;

impl MutationAuthorizer for ServerOnlyMutationAuthorizer {
    fn authorize(&self, _ctx: &MutationAuthContext, mutation: &QueuedMutation) -> MutationStatus {
        if mutation.connection_id.is_server() {
            MutationStatus::Ok
        } else {
            MutationStatus::Forbidden
        }
    }
}

/// Minimal representation of a component change event emitted by typed systems.
#[derive(Debug, Clone, Message)]
pub struct ComponentChangeEvent {
    pub entity: SerializableEntity,
    pub component_type: String,
    pub value: Vec<u8>,
}

/// Event emitted when an entity is despawned.
#[derive(Debug, Clone, Message)]
pub struct EntityDespawnEvent {
    pub entity: SerializableEntity,
}

fn apply_typed_mutation<T>(world: &mut World, mutation: &QueuedMutation) -> MutationStatus
where
    T: Component + serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + 'static + std::fmt::Debug,
{
    // Deserialize bincode bytes → concrete component type
    let value: T = match bincode::serde::decode_from_slice(&mutation.value, bincode::config::standard()) {
        Ok((v, _)) => v,
        Err(_err) => {
            return MutationStatus::ValidationError;
        }
    };

    bevy::log::info!("[apply_typed_mutation] Applying mutation: entity={:?}, type={}, value={:?}",
        mutation.entity, mutation.component_type, value);

    // Check if this is a request to spawn a new entity
    if mutation.entity == SerializableEntity::DANGLING {
        // Spawn a new entity with the component
        world.spawn(value);
        bevy::log::info!("[apply_typed_mutation] Spawned new entity with component {}", mutation.component_type);
        return MutationStatus::Ok;
    }

    let entity = mutation.entity.to_entity();
    match world.get_entity_mut(entity) {
        Ok(mut entity_mut) => {
            // Bevy's insert semantics: insert or replace the component value.
            entity_mut.insert(value);
            MutationStatus::Ok
        }
        Err(_) => MutationStatus::NotFound,
    }
}
fn snapshot_typed<T>(world: &mut World) -> Vec<(SerializableEntity, Vec<u8>)>
where
    T: Component + serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
{
    let mut results = Vec::new();

    // Use a temporary query to iterate all entities with this component type.
    let mut query = world.query::<(Entity, &T)>();
    for (entity, component) in query.iter(world) {
        // Serialize component directly to bincode bytes
        let bytes = bincode::serde::encode_to_vec(component, bincode::config::standard())
            .unwrap_or_default();
        results.push((SerializableEntity::from(entity), bytes));
    }

    results
}



/// Helper used by [`AppEventworkSyncExt::sync_component`] to register a type.
#[cfg(feature = "runtime")]
pub fn register_component<T>(app: &mut App, config: Option<ComponentSyncConfig>)
where
    T: Component + serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + 'static + std::fmt::Debug,
{
    // Register in SyncRegistry
    {
        let mut registry = app.world_mut().get_resource_or_insert_with(SyncRegistry::default);
        // Use short type name (just the struct name, no module path) for stability
        // This ensures client and server use the same type identifier
        let full_type_name = std::any::type_name::<T>();
        let type_name = full_type_name.rsplit("::").next().unwrap_or(full_type_name).to_string();
        registry.register_component(ComponentRegistration {
            type_id: std::any::TypeId::of::<T>(),
            type_name,
            config: config.unwrap_or_default(),
            apply_mutation: apply_typed_mutation::<T>,
            snapshot_all: snapshot_typed::<T>,
        });
    }

    // Add the typed system that will emit change events for this component type.
    crate::systems::register_component_system::<T>(app);
}

