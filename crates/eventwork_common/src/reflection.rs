use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use crate::NetworkMessage;

/// A message that contains entity component data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReflectedEntityData {
    /// The entity ID
    pub entity_id: u64,
    /// The component type name
    pub component_type: String,
    /// The serialized component data
    pub data: Vec<u8>,
}

impl NetworkMessage for ReflectedEntityData {
    const NAME: &'static str = "eventwork::ReflectedEntityData";
}

/// A request to subscribe to entity component changes
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscribeToComponent {
    /// The entity ID to subscribe to
    pub entity_id: u64,
    /// The component type name
    pub component_type: String,
    /// Optional field filter (dot notation path)
    pub field_path: Option<String>,
}

impl NetworkMessage for SubscribeToComponent {
    const NAME: &'static str = "eventwork::SubscribeToComponent";
}

/// A request to update an entity component (serialized version)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateEntityComponent {
    /// The entity ID to update
    pub entity_id: u64,
    /// The component type name
    pub component_type: String,
    /// The serialized component data
    pub data: Vec<u8>,
}

impl NetworkMessage for UpdateEntityComponent {
    const NAME: &'static str = "eventwork::UpdateEntityComponent";
}

/// A typed request to update an entity component
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct TypedUpdateEntityComponent<T> {
    /// The entity ID to update
    pub entity_id: u64,
    /// The component data
    pub component: T,
}

impl<T: NetworkMessage> NetworkMessage for TypedUpdateEntityComponent<T> {
    const NAME: &'static str = T::NAME;
}

/// A request to discover entities with specific components
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiscoverEntities {
    /// Optional component type filter (only return entities with these components)
    pub component_types: Vec<String>,
}

impl NetworkMessage for DiscoverEntities {
    const NAME: &'static str = "eventwork::DiscoverEntities";
}

/// Response containing discovered entity information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiscoveredEntities {
    /// List of discovered entities
    pub entities: Vec<EntityInfo>,
}

impl NetworkMessage for DiscoveredEntities {
    const NAME: &'static str = "eventwork::DiscoveredEntities";
}

/// Information about a discovered entity
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EntityInfo {
    /// Entity ID
    pub id: u64,
    /// List of component types on this entity
    pub component_types: Vec<String>,
}

/// A request to subscribe to entity creation/deletion events
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscribeToEntityChanges {
    /// Optional component type filter (only notify about entities with these components)
    pub component_types: Vec<String>,
}

impl NetworkMessage for SubscribeToEntityChanges {
    const NAME: &'static str = "eventwork::SubscribeToEntityChanges";
}

/// Notification about entity creation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EntityCreated {
    /// Information about the created entity
    pub entity: EntityInfo,
}

impl NetworkMessage for EntityCreated {
    const NAME: &'static str = "eventwork::EntityCreated";
}

/// Notification about entity deletion
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EntityDeleted {
    /// ID of the deleted entity
    pub entity_id: u64,
}

impl NetworkMessage for EntityDeleted {
    const NAME: &'static str = "eventwork::EntityDeleted";
}

/// Error response for reflection operations
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReflectionError {
    /// The entity ID related to the error (if applicable)
    pub entity_id: Option<u64>,
    /// The component type related to the error (if applicable)
    pub component_type: Option<String>,
    /// Error code
    pub error_code: ReflectionErrorCode,
    /// Human-readable error message
    pub message: String,
}

impl NetworkMessage for ReflectionError {
    const NAME: &'static str = "eventwork::ReflectionError";
}

/// Error codes for reflection operations
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ReflectionErrorCode {
    /// Entity not found
    EntityNotFound,
    /// Component not found on entity
    ComponentNotFound,
    /// Field path not found in component
    FieldPathNotFound,
    /// Validation failed for component update
    ValidationFailed,
    /// Serialization/deserialization error
    SerializationError,
    /// Other error
    Other,
}


