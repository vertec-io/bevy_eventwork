use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::SyncError;
use crate::traits::SyncComponent;

/// Registry mapping component type names to deserializer functions.
///
/// This is the client-side type registry that allows deserializing component data
/// received from the server. Each component type must be registered before it can
/// be deserialized.
///
/// # Example
///
/// ```rust,ignore
/// use eventwork_client::{ClientRegistryBuilder, SyncComponent};
///
/// let registry = ClientRegistryBuilder::new()
///     .register::<Position>()
///     .register::<Velocity>()
///     .build();
/// ```
pub struct ClientRegistry {
    /// Map from component type name to deserializer function
    deserializers: HashMap<String, Arc<dyn Fn(&[u8]) -> Result<Box<dyn Any + Send + Sync>, bincode::error::DecodeError> + Send + Sync>>,
    /// Map from component type name to TypeId (for type checking)
    type_ids: HashMap<String, TypeId>,
}

impl ClientRegistry {
    /// Create a new empty registry.
    ///
    /// Most users should use `ClientRegistryBuilder` instead.
    pub fn new() -> Self {
        Self {
            deserializers: HashMap::new(),
            type_ids: HashMap::new(),
        }
    }

    /// Register a component type.
    ///
    /// This creates a deserializer function that can convert bincode bytes
    /// into the concrete component type.
    pub fn register<T: SyncComponent>(&mut self) {
        let name = T::component_name().to_string();

        self.type_ids.insert(name.clone(), TypeId::of::<T>());

        self.deserializers.insert(
            name,
            Arc::new(|data| {
                let (component, _) = bincode::serde::decode_from_slice::<T, _>(data, bincode::config::standard())?;
                Ok(Box::new(component) as Box<dyn Any + Send + Sync>)
            }),
        );
    }

    /// Deserialize component data into the concrete type T.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::TypeNotRegistered` if the component type is not registered.
    /// Returns `SyncError::DeserializationFailed` if deserialization fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let position: Position = registry.deserialize("Position", &bytes)?;
    /// ```
    pub fn deserialize<T: 'static>(&self, name: &str, data: &[u8]) -> Result<T, SyncError> {
        let deserializer = self.deserializers.get(name)
            .ok_or_else(|| SyncError::TypeNotRegistered {
                component_name: name.to_string(),
            })?;

        let boxed = deserializer(data)
            .map_err(|e: bincode::error::DecodeError| SyncError::DeserializationFailed {
                component_name: name.to_string(),
                error: format!("{:?}", e),
            })?;

        boxed.downcast::<T>()
            .map(|b| *b)
            .map_err(|_| SyncError::DeserializationFailed {
                component_name: name.to_string(),
                error: "Type mismatch".to_string(),
            })
    }

    /// Check if a component type is registered.
    pub fn is_registered(&self, name: &str) -> bool {
        self.deserializers.contains_key(name)
    }

    /// Get the TypeId for a registered component type.
    pub fn get_type_id(&self, name: &str) -> Option<TypeId> {
        self.type_ids.get(name).copied()
    }
}

impl Default for ClientRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing a `ClientRegistry`.
///
/// This provides a fluent API for registering multiple component types.
///
/// # Example
///
/// ```rust,ignore
/// use eventwork_client::ClientRegistryBuilder;
///
/// let registry = ClientRegistryBuilder::new()
///     .register::<Position>()
///     .register::<Velocity>()
///     .register::<Health>()
///     .build();
/// ```
pub struct ClientRegistryBuilder {
    registry: ClientRegistry,
}

impl ClientRegistryBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            registry: ClientRegistry::new(),
        }
    }

    /// Register a component type.
    ///
    /// This can be chained to register multiple types.
    pub fn register<T: SyncComponent>(mut self) -> Self {
        self.registry.register::<T>();
        self
    }

    /// Build the final `ClientRegistry` wrapped in an `Arc`.
    ///
    /// The registry is wrapped in an Arc because it needs to be shared
    /// across multiple reactive contexts.
    pub fn build(self) -> Arc<ClientRegistry> {
        Arc::new(self.registry)
    }
}

impl Default for ClientRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

