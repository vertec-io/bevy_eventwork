use serde::{de::DeserializeOwned, Serialize};

/// Trait for types that can be synchronized via eventwork_sync.
///
/// This trait identifies component types that can be subscribed to and received
/// from an eventwork_sync server. Types implementing this trait must be serializable
/// and deserializable using serde.
///
/// # Type Identification
///
/// Components are identified by their **short type name** (struct name only, no module path).
/// This matches the server-side behavior in eventwork_sync and provides stability across
/// module refactoring.
///
/// # Example
///
/// ```rust,ignore
/// use serde::{Serialize, Deserialize};
/// use eventwork_client::SyncComponent;
///
/// #[derive(Serialize, Deserialize, Clone, Debug)]
/// struct Position {
///     x: f32,
///     y: f32,
/// }
///
/// impl SyncComponent for Position {
///     fn component_name() -> &'static str {
///         "Position"  // Short name only, no module path
///     }
/// }
/// ```
///
/// # Requirements
///
/// - Must implement `Serialize` and `DeserializeOwned` from serde
/// - Must be `Send + Sync + 'static` for use in reactive contexts
/// - Component name must match the server-side registration
pub trait SyncComponent: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// Returns the component type name used for synchronization.
    ///
    /// This should be the **short type name** (struct name only, no module path)
    /// to match the server-side behavior.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// impl SyncComponent for Position {
    ///     fn component_name() -> &'static str {
    ///         "Position"  // Correct: short name
    ///         // NOT "my_game::components::Position"
    ///     }
    /// }
    /// ```
    fn component_name() -> &'static str;
}

/// Helper macro to automatically implement SyncComponent using the type's short name.
///
/// This macro extracts the short type name at compile time and implements the
/// SyncComponent trait.
///
/// # Example
///
/// ```rust,ignore
/// use serde::{Serialize, Deserialize};
/// use eventwork_client::impl_sync_component;
///
/// #[derive(Serialize, Deserialize, Clone, Debug)]
/// struct Position {
///     x: f32,
///     y: f32,
/// }
///
/// impl_sync_component!(Position);
/// ```
#[macro_export]
macro_rules! impl_sync_component {
    ($type:ty) => {
        impl $crate::SyncComponent for $type {
            fn component_name() -> &'static str {
                // Extract short name from full type path at runtime
                let full_name = std::any::type_name::<$type>();
                // Find the last "::" and return everything after it
                full_name.rsplit("::").next().unwrap_or(full_name)
            }
        }
    };
}

