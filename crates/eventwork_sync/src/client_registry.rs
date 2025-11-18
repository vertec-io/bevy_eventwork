//! Client-side type registry for deserializing and serializing component data.
//!
//! This module provides a registry that maps component type names to
//! deserializer and serializer functions. This allows client applications
//! (web UI, native tools, etc.) to work with arbitrary component data
//! without knowing the concrete types at compile time.
//!
//! This is NOT needed on the server side - the server uses Bevy's type registry
//! and reflection system. This is specifically for clients that need to
//! deserialize component data for display or serialize mutations to send back.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A function that can deserialize bincode bytes into a serde_json::Value.
pub type DeserializeFn = fn(&[u8]) -> Result<serde_json::Value, bincode::error::DecodeError>;

/// A function that can serialize a serde_json::Value into bincode bytes.
pub type SerializeFn = fn(&serde_json::Value) -> Result<Vec<u8>, bincode::error::EncodeError>;

/// Registry mapping component type names to deserializer and serializer functions.
///
/// This is the client-side equivalent of Bevy's type registry. It allows clients
/// to work with component data in a type-safe way without compile-time knowledge
/// of all component types.
///
/// # Example
/// ```ignore
/// use eventwork_sync::client_registry::ComponentTypeRegistry;
///
/// let mut registry = ComponentTypeRegistry::new();
/// registry.register::<MyComponent>();
///
/// // Deserialize component data from server
/// let json_value = registry.deserialize_by_name("MyComponent", &bytes)?;
///
/// // Serialize mutation to send to server
/// let bytes = registry.serialize_by_name("MyComponent", &json_value)?;
/// ```
#[derive(Clone)]
pub struct ComponentTypeRegistry {
    deserializers: HashMap<String, DeserializeFn>,
    serializers: HashMap<String, SerializeFn>,
}

impl ComponentTypeRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            deserializers: HashMap::new(),
            serializers: HashMap::new(),
        }
    }

    /// Register a component type with the registry.
    ///
    /// This creates deserializer and serializer functions that:
    /// - Deserialize: bincode bytes → concrete type T → serde_json::Value for display
    /// - Serialize: serde_json::Value → concrete type T → bincode bytes for mutations
    ///
    /// The type name used for registration is the short name (struct name only,
    /// no module path) to match the server-side behavior.
    ///
    /// # Example
    /// ```ignore
    /// let mut registry = ComponentTypeRegistry::new();
    /// registry.register::<DemoCounter>();
    /// ```
    pub fn register<T>(&mut self)
    where
        T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + 'static,
    {
        // Use short type name (just the struct name, no module path) for stability
        // This matches what eventwork_sync uses on the server side
        let full_type_name = std::any::type_name::<T>();
        let type_name = full_type_name.rsplit("::").next().unwrap_or(full_type_name).to_string();

        let deserializer: DeserializeFn = |bytes: &[u8]| {
            // Deserialize bincode bytes to concrete type T
            let (value, _): (T, _) = bincode::serde::decode_from_slice(bytes, bincode::config::standard())?;
            // Convert to JSON for display in UI
            serde_json::to_value(value)
                .map_err(|_| bincode::error::DecodeError::OtherString("JSON conversion failed".into()))
        };

        let serializer: SerializeFn = |json_value: &serde_json::Value| {
            // Convert JSON to concrete type T
            let value: T = serde_json::from_value(json_value.clone())
                .map_err(|_| bincode::error::EncodeError::OtherString("JSON to type conversion failed".into()))?;

            // Serialize to bincode bytes
            bincode::serde::encode_to_vec(&value, bincode::config::standard())
        };

        self.deserializers.insert(type_name.clone(), deserializer);
        self.serializers.insert(type_name, serializer);
    }

    /// Deserialize component data by type name.
    ///
    /// Returns Ok(serde_json::Value) if the type is registered and deserialization succeeds.
    /// Returns Err if the type is not registered or deserialization fails.
    pub fn deserialize_by_name(
        &self,
        type_name: &str,
        bytes: &[u8],
    ) -> Result<serde_json::Value, DeserializeError> {
        let deserializer = self.deserializers.get(type_name)
            .ok_or_else(|| DeserializeError::TypeNotRegistered(type_name.to_string()))?;

        deserializer(bytes)
            .map_err(|e| DeserializeError::BincodeError(format!("{:?}", e)))
    }

    /// Serialize component data by type name.
    ///
    /// Returns Ok(Vec<u8>) if the type is registered and serialization succeeds.
    /// Returns Err if the type is not registered or serialization fails.
    pub fn serialize_by_name(
        &self,
        type_name: &str,
        json_value: &serde_json::Value,
    ) -> Result<Vec<u8>, SerializeError> {
        let serializer = self.serializers.get(type_name)
            .ok_or_else(|| SerializeError::TypeNotRegistered(type_name.to_string()))?;

        serializer(json_value)
            .map_err(|e| SerializeError::BincodeError(format!("{:?}", e)))
    }

    /// Check if a type is registered.
    pub fn has_type(&self, type_name: &str) -> bool {
        self.deserializers.contains_key(type_name)
    }

    /// Get all registered type names.
    pub fn registered_types(&self) -> Vec<String> {
        self.deserializers.keys().cloned().collect()
    }
}

impl Default for ComponentTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during deserialization.
#[derive(Debug, Clone)]
pub enum DeserializeError {
    /// The type name is not registered in the registry.
    TypeNotRegistered(String),
    /// Bincode deserialization failed.
    BincodeError(String),
}

impl std::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeserializeError::TypeNotRegistered(type_name) => {
                write!(f, "Type '{}' not registered in ComponentTypeRegistry", type_name)
            }
            DeserializeError::BincodeError(e) => {
                write!(f, "Bincode deserialization error: {}", e)
            }
        }
    }
}

impl std::error::Error for DeserializeError {}

/// Errors that can occur during serialization.
#[derive(Debug, Clone)]
pub enum SerializeError {
    /// The type name is not registered in the registry.
    TypeNotRegistered(String),
    /// Bincode serialization failed.
    BincodeError(String),
}

impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializeError::TypeNotRegistered(type_name) => {
                write!(f, "Type '{}' not registered in ComponentTypeRegistry", type_name)
            }
            SerializeError::BincodeError(e) => {
                write!(f, "Bincode serialization error: {}", e)
            }
        }
    }
}

impl std::error::Error for SerializeError {}

