use bevy::prelude::*;
use eventwork_common::NetworkMessage;
use serde::{DeserializeOwned, Serialize};
use crate::reflection::*;
use crate::{NetworkProvider, OutboundMessage};
use eventwork_common::reflection::{DiscoverEntities, DiscoveredEntities, EntityCreated, EntityDeleted, ReflectedEntityData, SubscribeToComponent, SubscribeToEntityChanges, TypedUpdateEntityComponent, UpdateEntityComponent};
use std::marker::PhantomData;

/// Plugin for networked reflection capabilities
pub struct NetworkedReflectionPlugin<NP: NetworkProvider> {
    _marker: PhantomData<NP>,
}

impl<NP: NetworkProvider> Default for NetworkedReflectionPlugin<NP> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<NP: NetworkProvider> Plugin for NetworkedReflectionPlugin<NP> {
    fn build(&self, app: &mut App) {
        app.init_resource::<ComponentSubscriptions>()
           .init_resource::<EntityChangeSubscriptions>()
           .init_resource::<ComponentHandlerRegistry>()
           .add_systems(Update, (
               handle_component_subscriptions::<NP>,
               handle_component_updates::<NP>,
               handle_entity_discovery::<NP>,
               handle_entity_change_subscriptions::<NP>,
               notify_entity_creation::<NP>,
               notify_entity_deletion::<NP>,
           ));
           
        // Register the network messages
        app.listen_for_message::<SubscribeToComponent, NP>();
        app.listen_for_message::<UpdateEntityComponent, NP>();
        app.listen_for_message::<DiscoverEntities, NP>();
        app.listen_for_message::<SubscribeToEntityChanges, NP>();
        
        // Register outbound messages
        app.register_outbound_message::<ReflectedEntityData, NP>();
        app.register_outbound_message::<DiscoveredEntities, NP>();
        app.register_outbound_message::<EntityCreated, NP>();
        app.register_outbound_message::<EntityDeleted, NP>();
    }
}

/// Extension trait to register component types for networked reflection
pub trait NetworkedReflectionExt {
    /// Register a component type for networked reflection
    fn register_networked_component<T: Component + NetworkMessage + Clone + 'static>(&mut self) -> &mut Self;
}

impl NetworkedReflectionExt for App {
    fn register_networked_component<T: Component + NetworkMessage + Clone + 'static>(&mut self) -> &mut Self {
        // Register the component with the handler registry
        {
            let mut registry = self.world_mut().resource_mut::<ComponentHandlerRegistry>();
            registry.register::<T>();
        }
        
        // Register the typed update handler for this specific component type
        self.listen_for_message::<TypedUpdateEntityComponent<T>, NP: NetworkProvider>();
        self.add_systems(Update, handle_typed_component_updates::<T, NP: NetworkProvider>);
        self.add_systems(Update, handle_component_updates::<T, NP: NetworkProvider>);
        
        // Add a system to detect changes and notify subscribers
        self.add_systems(Update, move |
            changed_components: Query<(Entity, &T), Changed<T>>,
            subscriptions: Res<ComponentSubscriptions>,
            mut outbound: EventWriter<OutboundMessage<ReflectedEntityData>>,
        | {
            for (entity, component) in changed_components.iter() {
                let entity_id = entity.to_bits();
                
                // Get subscribers for this entity and component type
                let subscribers = subscriptions.get_subscribers(entity_id, T::NAME);
                
                if !subscribers.is_empty() {
                    // Serialize the component data
                    if let Ok(data) = bincode::serialize(component) {
                        // Send the data to all subscribers
                        for client_id in subscribers {
                            let notification = ReflectedEntityData {
                                entity_id,
                                component_type: T::NAME.to_string(),
                                data: data.clone(),
                                field_path: None, // No field path filtering
                            };
                            
                            outbound.send(OutboundMessage::new(ReflectedEntityData::NAME.to_string(), notification).for_client(client_id));
                        }
                    }
                }
            }
        });
        
        self
    }
}




