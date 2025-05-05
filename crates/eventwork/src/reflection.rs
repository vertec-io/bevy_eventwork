use bevy::prelude::*;
use bevy::reflect::{Reflect, TypeRegistry, ReflectRef, FromReflect};
use eventwork_common::reflection::{DiscoverEntities, DiscoveredEntities, EntityCreated, EntityDeleted, EntityInfo, ReflectedEntityData, ReflectionError, ReflectionErrorCode, SubscribeToComponent, SubscribeToEntityChanges, TypedUpdateEntityComponent, UpdateEntityComponent};
use eventwork_common::NetworkMessage;
use crate::{ConnectionId, Network, NetworkData, NetworkProvider, OutboundMessage};
use std::collections::HashMap;
use std::any::TypeId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Helper function to extract a field value using Bevy's reflection system
// fn extract_field_by_path<T: Reflect>(component: &T, path: &str, type_registry: &TypeRegistry) -> Option<Box<dyn Reflect>> {
//     // Start with the component itself
//     let mut current_value: &dyn Reflect = component;
    
//     // Split the path into parts
//     for part in path.split('.') {
//         // Try to access the field based on the current value's type
//         match current_value.reflect_ref() {
//             ReflectRef::Struct(struct_value) => {
//                 // Access a field in a struct
//                 if let Some(field_value) = struct_value.field(part) {
//                     current_value = field_value;
//                 } else {
//                     return None; // Field not found
//                 }
//             }
//             ReflectRef::List(list_value) => {
//                 // Access an element in a list by index
//                 if let Ok(index) = part.parse::<usize>() {
//                     if let Some(element) = list_value.get(index) {
//                         current_value = element;
//                     } else {
//                         return None; // Index out of bounds
//                     }
//                 } else {
//                     return None; // Invalid index
//                 }
//             }
//             ReflectRef::Map(map_value) => {
//                 // Access a value in a map by key
//                 // Note: This assumes string keys, which may not always be the case
//                 let key = part.to_string();
//                 if let Some(value) = map_value.get(&key) {
//                     current_value = value;
//                 } else {
//                     return None; // Key not found
//                 }
//             }
//             _ => return None, // Can't access fields on this type
//         }
//     }
    
//     // Clone the final value
//     current_value.clone_value()
// }

/// Resource to track component subscriptions
#[derive(Resource, Default)]
pub struct ComponentSubscriptions {
    /// Map of entity ID -> component type -> subscribers
    subscriptions: HashMap<u64, HashMap<String, Vec<ConnectionId>>>,
}

impl ComponentSubscriptions {
    /// Add a subscription
    pub fn add_subscription(&mut self, entity_id: u64, component_type: String, client_id: ConnectionId) {
        let entity_subs = self.subscriptions.entry(entity_id).or_default();
        let component_subs = entity_subs.entry(component_type).or_default();
        
        // Don't add duplicate subscriptions
        if !component_subs.contains(&client_id) {
            component_subs.push(client_id);
        }
    }
    
    /// Get subscribers for an entity component
    pub fn get_subscribers(&self, entity_id: u64, component_type: &str) -> Vec<ConnectionId> {
        self.subscriptions
            .get(&entity_id)
            .and_then(|entity_subs| entity_subs.get(component_type))
            .map(|subs| subs.clone())
            .unwrap_or_default()
    }
}

/// Resource to track component handlers
#[derive(Resource, Default)]
pub struct ComponentHandlerRegistry {
    /// Map of component type name -> getter function
    getters: HashMap<&'static str, Box<dyn Fn(&EntityRef) -> Option<Vec<u8>> + Send + Sync>>,
    /// Map of component type name -> setter function
    setters: HashMap<&'static str, Box<dyn Fn(Entity, &[u8], &mut Commands) + Send + Sync>>,
}

impl ComponentHandlerRegistry {
    /// Register a component type with reflection support
    pub fn register_reflected_component<T: Component + NetworkMessage + Reflect + FromReflect + 'static>(&mut self) {
        // Register standard getter/setter
        self.getters.insert(T::NAME, Box::new(|entity_ref| {
            entity_ref.get::<T>().and_then(|component| {
                bincode::serialize(component).ok()
            })
        }));
        
        self.setters.insert(T::NAME, Box::new(|entity, data, commands| {
            if let Ok(component) = bincode::deserialize::<T>(data) {
                commands.entity(entity).insert(component);
            }
        }));
    }
    
    /// Get component data
    pub fn get_component_data(&self, entity: &EntityRef, component_type: &str) -> Option<Vec<u8>> {
        self.getters.get(component_type).and_then(|getter| getter(entity))
    }
    
    /// Update component data
    pub fn update_component_data(&self, entity: Entity, component_type: &str, data: &[u8], commands: &mut Commands) {
        if let Some(setter) = self.setters.get(component_type) {
            setter(entity, data, commands);
        }
    }
}

/// Resource to track component validators
#[derive(Resource, Default)]
pub struct ComponentValidatorRegistry {
    /// Map of component type name -> validator function
    validators: HashMap<&'static str, Box<dyn Fn(Entity, &[u8], &World) -> bool + Send + Sync>>,
    /// Map of component type name -> typed validator function
    typed_validators: HashMap<TypeId, Box<dyn Fn(Entity, &dyn std::any::Any, &World) -> bool + Send + Sync>>,
}

impl ComponentValidatorRegistry {
    /// Register a validator for a component type
    pub fn register_validator<T: Component + NetworkMessage + 'static>(
        &mut self,
        validator: impl Fn(Entity, &T, &World) -> bool + Send + Sync + 'static
    ) {
        // Wrap the validator in an Arc so we can share it
        let validator = Arc::new(validator);
        let validator_clone = validator.clone();
        
        // Register serialized validator
        self.validators.insert(T::NAME, Box::new(move |entity, data, world| {
            if let Ok(component) = bincode::deserialize::<T>(data) {
                validator(entity, &component, world)
            } else {
                false // Deserialization failed, reject the update
            }
        }));
        
        // Register typed validator
        self.typed_validators.insert(TypeId::of::<T>(), Box::new(move |entity, component, world| {
            if let Some(component) = component.downcast_ref::<T>() {
                validator_clone(entity, component, world)
            } else {
                false // Type mismatch, reject the update
            }
        }));
    }
    
    /// Validate component update
    pub fn validate_update(&self, entity: Entity, component_type: &str, data: &[u8], world: &World) -> bool {
        self.validators
            .get(component_type)
            .map_or(true, |validator| validator(entity, data, world))
    }
    
    /// Validate typed component update
    pub fn validate_typed_update<T: 'static>(&self, entity: Entity, component: &T, world: &World) -> bool {
        self.typed_validators
            .get(&TypeId::of::<T>())
            .map_or(true, |validator| validator(entity, component, world))
    }
}

/// System to handle component subscription requests
pub fn handle_component_subscriptions<NP: NetworkProvider>(
    mut requests: EventReader<NetworkData<SubscribeToComponent>>,
    mut subscriptions: ResMut<ComponentSubscriptions>,
    registry: Res<ComponentHandlerRegistry>,
    world: &World,
    mut outbound: EventWriter<OutboundMessage<ReflectedEntityData>>,
    mut error_outbound: EventWriter<OutboundMessage<ReflectionError>>,
) {
    for request in requests.read() {
        let client_id = request.source();
        let entity_id = request.entity_id;
        let component_type = request.component_type.clone();
        
        // Add the subscription
        subscriptions.add_subscription(entity_id, component_type.clone(), *client_id);
        
        // Send the current state immediately if the entity exists
        let entity = Entity::from_bits(entity_id);
        if let Ok(entity_ref) = world.get_entity(entity) {
            let data = registry.get_component_data(&entity_ref, &component_type);
            
            if let Some(data) = data {
                // Send the component data to the client
                let response = ReflectedEntityData {
                    entity_id,
                    component_type: component_type.clone(),
                    data,
                };
                
                outbound.send(OutboundMessage::new(ReflectedEntityData::NAME.to_string(), response).for_client(*client_id));
            } else {
                // Component not found, send an error
                let error = ReflectionError {
                    entity_id: Some(entity_id),
                    component_type: Some(component_type.clone()),
                    error_code: ReflectionErrorCode::ComponentNotFound,
                    message: format!("Component {} not found on entity {}", component_type, entity_id),
                };
                
                error_outbound.send(OutboundMessage::new(ReflectionError::NAME.to_string(), error).for_client(*client_id));
            }
        } else {
            // Entity not found, send an error
            let error = ReflectionError {
                entity_id: Some(entity_id),
                component_type: Some(component_type.clone()),
                error_code: ReflectionErrorCode::EntityNotFound,
                message: format!("Entity {} not found", entity_id),
            };
            
            error_outbound.send(OutboundMessage::new(ReflectionError::NAME.to_string(), error).for_client(*client_id));
        }
    }
}

/// System to handle component updates with validation and error reporting
pub fn handle_component_updates<NP: NetworkProvider>(
    mut requests: EventReader<NetworkData<UpdateEntityComponent>>,
    registry: Res<ComponentHandlerRegistry>,
    validators: Res<ComponentValidatorRegistry>,
    world: &World,
    mut commands: Commands,
    mut error_outbound: EventWriter<OutboundMessage<ReflectionError>>,
) {
    for request in requests.read() {
        let client_id = request.source();
        let entity_id = request.entity_id;
        let component_type = &request.component_type;
        let data = &request.data;
        let entity = Entity::from_bits(entity_id);
        
        // Check if entity exists
        if world.get_entity(entity).is_err() {
            let error = ReflectionError {
                entity_id: Some(entity_id),
                component_type: Some(component_type.clone()),
                error_code: ReflectionErrorCode::EntityNotFound,
                message: format!("Entity {} not found", entity_id),
            };
            
            error_outbound.send(OutboundMessage::new(ReflectionError::NAME.to_string(), error).for_client(*client_id));
            continue;
        }
        
        // Validate the update
        if validators.validate_update(entity, component_type, data, world) {
            // Use the registry to update the component
            registry.update_component_data(entity, component_type, data, &mut commands);
        } else {
            // Send a rejection message back to the client
            let error = ReflectionError {
                entity_id: Some(entity_id),
                component_type: Some(component_type.clone()),
                error_code: ReflectionErrorCode::ValidationFailed,
                message: format!("Validation failed for component update on entity {}", entity_id),
            };
            
            error_outbound.send(OutboundMessage::new(ReflectionError::NAME.to_string(), error).for_client(*client_id));
        }
    }
}

/// System to handle typed component updates
pub fn handle_typed_component_updates<T: Component + NetworkMessage + Clone, NP: NetworkProvider>(
    mut requests: EventReader<NetworkData<TypedUpdateEntityComponent<T>>>,
    validators: Res<ComponentValidatorRegistry>,
    world: &World,
    mut commands: Commands,
) {
    for request in requests.read() {
        let entity_id = request.entity_id;
        let component = &request.component;
        let entity = Entity::from_bits(entity_id);
        
        // Validate the update
        if validators.validate_typed_update(entity, component, world) {
            // Update the component directly without serialization/deserialization
            commands.entity(entity).insert(component.clone());
        } else {
            // Optionally, send a rejection message back to the client
        }
    }
}
    
/// Resource to track entity change subscriptions
#[derive(Resource, Default)]
pub struct EntityChangeSubscriptions {
    /// Map of client ID -> component type filters
    subscriptions: HashMap<ConnectionId, Vec<String>>,
}

impl EntityChangeSubscriptions {
    /// Add a subscription
    pub fn add_subscription(&mut self, client_id: ConnectionId, component_types: Vec<String>) {
        self.subscriptions.insert(client_id, component_types);
    }
    
    /// Get subscribers that match the given component types
    pub fn get_matching_subscribers(&self, component_types: &[&str]) -> Vec<ConnectionId> {
        self.subscriptions
            .iter()
            .filter_map(|(client_id, filter_types)| {
                // If the filter is empty, match all entities
                if filter_types.is_empty() {
                    return Some(*client_id);
                }
                
                // Otherwise, check if any of the entity's components match the filter
                for filter_type in filter_types {
                    if component_types.iter().any(|&t| t == filter_type) {
                        return Some(*client_id);
                    }
                }
                
                None
            })
            .collect()
    }
}

/// System to handle entity discovery requests
pub fn handle_entity_discovery<NP: NetworkProvider>(
    mut requests: EventReader<NetworkData<DiscoverEntities>>,
    world: &World,
    mut outbound: EventWriter<OutboundMessage<DiscoveredEntities>>,
) {
    for request in requests.read() {
        let client_id = request.source();
        let component_filters = &request.component_types;
        
        let mut discovered_entities = Vec::new();
        
        // Iterate through all entities in the world
        for entity_ref in world.iter_entities() {
            let entity_id = entity_ref.id().to_bits();
            let mut entity_component_types = Vec::new();
            
            // Get all component types for this entity
            for component_id in entity_ref.archetype().components() {
                if let Some(info) = world.components().get_info(component_id) {
                    let type_name = info.name();
                    entity_component_types.push(type_name.to_string());
                }
            }
            
            // Check if this entity matches the filter
            let matches_filter = component_filters.is_empty() || 
                component_filters.iter().any(|filter| 
                    entity_component_types.iter().any(|t| t == filter)
                );
            
            if matches_filter {
                discovered_entities.push(EntityInfo {
                    id: entity_id,
                    component_types: entity_component_types,
                });
            }
        }
        
        // Send the response
        let response = DiscoveredEntities {
            entities: discovered_entities,
        };
        
        outbound.send(OutboundMessage::new(DiscoveredEntities::NAME.to_string(), response).for_client(*client_id));
    }
}

/// System to handle entity change subscription requests
pub fn handle_entity_change_subscriptions<NP: NetworkProvider>(
    mut requests: EventReader<NetworkData<SubscribeToEntityChanges>>,
    mut subscriptions: ResMut<EntityChangeSubscriptions>,
) {
    for request in requests.read() {
        let client_id = request.source();
        let component_types = request.component_types.clone();
        
        // Add the subscription
        subscriptions.add_subscription(*client_id, component_types);
    }
}

/// System to detect entity creation and notify subscribers
pub fn notify_entity_creation<NP: NetworkProvider>(
    query: Query<Entity, Added<Transform>>, // Using Transform as a proxy for "newly spawned entities"
    world: &World,
    subscriptions: Res<EntityChangeSubscriptions>,
    mut outbound: EventWriter<OutboundMessage<EntityCreated>>,
) {
    for entity in query.iter() {
        let entity_id = entity.to_bits();
        let mut component_types = Vec::new();
        
        // Get all component types for this entity
        if let Ok(entity_ref) = world.get_entity(entity) {
            for component_id in entity_ref.archetype().components() {
                if let Some(info) = world.components().get_info(component_id) {
                    let type_name = info.name();
                    component_types.push(type_name.to_string());
                }
            }
        }
        
        // Get component types as string slices for matching
        let component_type_slices: Vec<&str> = component_types.iter()
            .map(|s| s.as_str())
            .collect();
        
        // Find subscribers that match this entity's components
        let matching_subscribers = subscriptions.get_matching_subscribers(&component_type_slices);
        
        // Notify subscribers
        for client_id in matching_subscribers {
            let notification = EntityCreated {
                entity: EntityInfo {
                    id: entity_id,
                    component_types: component_types.clone(),
                },
            };
            
            outbound.send(OutboundMessage::new(EntityCreated::NAME.to_string(), notification).for_client(client_id));
        }
    }
}

/// System to detect entity deletion and notify subscribers
pub fn notify_entity_deletion<NP: NetworkProvider>(
    mut removed: RemovedComponents<Transform>, // Using Transform as a proxy for "deleted entities"
    subscriptions: Res<EntityChangeSubscriptions>,
    mut outbound: EventWriter<OutboundMessage<EntityDeleted>>,
) {
    for entity in removed.read() {
        let entity_id = entity.to_bits();
        
        // Notify all subscribers (we can't filter by component type for deleted entities)
        for (client_id, _) in subscriptions.subscriptions.iter() {
            let notification = EntityDeleted {
                entity_id,
            };
            
            outbound.send(OutboundMessage::new(EntityDeleted::NAME.to_string(), notification).for_client(*client_id));
        }
    }
}

// /// Resource for batching component updates
// #[derive(Resource)]
// pub struct ComponentUpdateBatcher {
//     /// Map of client ID -> entity ID -> component type -> pending update
//     pending_updates: HashMap<ConnectionId, HashMap<u64, HashMap<String, ReflectedEntityData>>>,
//     /// Timer for flushing batched updates
//     batch_timer: Timer,
// }

// impl Default for ComponentUpdateBatcher {
//     fn default() -> Self {
//         Self {
//             pending_updates: HashMap::new(),
//             batch_timer: Timer::new(Duration::from_millis(50), TimerMode::Repeating),
//         }
//     }
// }

// impl ComponentUpdateBatcher {
//     /// Queue an update for batching
//     pub fn queue_update(&mut self, client_id: ConnectionId, update: ReflectedEntityData) {
//         let client_updates = self.pending_updates.entry(client_id).or_default();
//         let entity_updates = client_updates.entry(update.entity_id).or_default();
//         entity_updates.insert(update.component_type.clone(), update);
//     }
    
//     /// Flush all pending updates for a client
//     pub fn flush_updates(&mut self, client_id: ConnectionId) -> Option<BatchedEntityUpdates> {
//         if let Some(client_updates) = self.pending_updates.remove(&client_id) {
//             let mut updates = Vec::new();
            
//             for entity_updates in client_updates.values() {
//                 for update in entity_updates.values() {
//                     updates.push(update.clone());
//                 }
//             }
            
//             if !updates.is_empty() {
//                 return Some(BatchedEntityUpdates { updates });
//             }
//         }
        
//         None
//     }
    
//     /// Tick the batch timer and return clients that need updates flushed
//     pub fn tick(&mut self, delta: Duration) -> Vec<ConnectionId> {
//         self.batch_timer.tick(delta);
        
//         if self.batch_timer.just_finished() {
//             self.pending_updates.keys().cloned().collect()
//         } else {
//             Vec::new()
//         }
//     }
// }

// /// System to batch and send component updates
// pub fn batch_component_updates<NP: NetworkProvider>(
//     time: Res<Time>,
//     mut batcher: ResMut<ComponentUpdateBatcher>,
//     mut outbound: EventWriter<OutboundMessage<BatchedEntityUpdates>>,
// ) {
//     let clients_to_flush = batcher.tick(time.delta());
    
//     for client_id in clients_to_flush {
//         if let Some(batch) = batcher.flush_updates(client_id) {
//             outbound.send(OutboundMessage::new(BatchedEntityUpdates::NAME.to_string(), batch).for_client(client_id));
//         }
//     }
// }








