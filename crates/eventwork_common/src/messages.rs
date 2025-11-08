use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/*
/// Any type that should be sent over the wire has to implement [`NetworkMessage`].
///
/// ## Example
/// ```rust
/// use bevy_eventwork::NetworkMessage;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct PlayerInformation {
///     health: usize,
///     position: (u32, u32, u32)
/// }
///
/// impl NetworkMessage for PlayerInformation {
///     const NAME: &'static str = "PlayerInfo";
/// }
/// ```


*/
/// Marks a type as an eventwork message with an explicit name
///
/// This trait is used when you need explicit control over the message name,
/// such as for versioned protocols, cross-language communication, or when
/// you need human-readable message names.
///
/// For automatic message registration without boilerplate, see [`EventworkMessage`].
///
/// ## Example
/// ```rust
/// use eventwork_common::NetworkMessage;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct PlayerInformation {
///     health: usize,
///     position: (u32, u32, u32)
/// }
///
/// impl NetworkMessage for PlayerInformation {
///     const NAME: &'static str = "game:v1:PlayerInfo";
/// }
/// ```
pub trait NetworkMessage: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// A unique name to identify your message, this needs to be unique __across all included crates__
    ///
    /// A good combination is crate name + struct name, optionally with version.
    const NAME: &'static str;
}

/// Automatic network message with inferred type name
///
/// This trait is automatically implemented for all types that are
/// `Serialize + DeserializeOwned + Send + Sync + 'static`.
///
/// The type name is generated from `std::any::type_name()` and cached
/// for performance. The first access incurs a ~500ns cost, subsequent
/// accesses are ~50-100ns.
///
/// Use this trait when:
/// - Rapid prototyping
/// - Internal game state synchronization
/// - Working with external types (solves orphan rule)
/// - Developer ergonomics is priority
///
/// Use [`NetworkMessage`] when:
/// - Building public APIs or protocols
/// - Need version stability across updates
/// - Cross-language communication
/// - Need human-readable message names
///
/// ## Example
///
/// ```rust
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Clone)]
/// struct PlayerPosition {
///     x: f32,
///     y: f32,
///     z: f32,
/// }
///
/// // No trait implementation needed!
/// // EventworkMessage is automatically implemented.
/// // Use with app.register_network_message::<PlayerPosition, Provider>();
/// ```
pub trait EventworkMessage: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// Returns the type name for this message type.
    ///
    /// The name is generated from `std::any::type_name()` and cached
    /// in a global static for performance.
    fn type_name() -> &'static str {
        use std::any::{TypeId, type_name};
        use std::collections::HashMap;
        use std::sync::{Mutex, OnceLock};

        static CACHE: OnceLock<Mutex<HashMap<TypeId, &'static str>>> = OnceLock::new();
        let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

        let type_id = TypeId::of::<Self>();

        // Fast path: check cache without holding lock long
        {
            let cache_guard = cache.lock().unwrap();
            if let Some(&name) = cache_guard.get(&type_id) {
                return name;
            }
        }

        // Slow path: generate and cache
        let full_type_name = type_name::<Self>();
        let static_name = Box::leak(full_type_name.to_string().into_boxed_str());

        {
            let mut cache_guard = cache.lock().unwrap();
            cache_guard.insert(type_id, static_name);
        }

        static_name
    }
}

// Blanket implementation for all serializable types
impl<T> EventworkMessage for T
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static
{}

/// Internal trait that provides message kind for network transmission
///
/// This trait provides a unified way to get the message identifier string.
/// It is automatically implemented for types that implement `NetworkMessage`.
///
/// For types that don't implement `NetworkMessage`, use `EventworkMessage::type_name()` directly.
///
/// Users should not implement this trait directly.
pub trait AnyNetworkMessage: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// Returns the message kind string for wire protocol
    fn message_kind() -> String;
}

// Implementation for types with explicit NetworkMessage
impl<T> AnyNetworkMessage for T
where
    T: NetworkMessage,
{
    fn message_kind() -> String {
        String::from(T::NAME)
    }
}

// Note: We cannot add a blanket impl for all EventworkMessage types here
// because it would conflict with the NetworkMessage impl above.
// Instead, the API methods will handle both cases:
// - If T: NetworkMessage, use T::message_kind() (via AnyNetworkMessage)
// - If T: EventworkMessage (but not NetworkMessage), use T::type_name() directly

/// Marks a type as a request type.
pub trait RequestMessage:
    Clone + Serialize + DeserializeOwned + Send + Sync + Debug + 'static
{
    /// The response type for the request.
    type ResponseMessage: NetworkMessage
        + Clone
        + Serialize
        + DeserializeOwned
        + Send
        + Sync
        + Debug
        + 'static;

    /// The label used for the request type, same rules as [`NetworkMessage`] in terms of naming.
    const REQUEST_NAME: &'static str;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "T: NetworkMessage")]
pub struct TargetedMessage<T: NetworkMessage> {
    pub target_id: String,
    pub message: T,
}

impl<T: NetworkMessage> NetworkMessage for TargetedMessage<T> {
    const NAME: &'static str = "eventwork::TargetedMessage";
}

impl<T: NetworkMessage> TargetedMessage<T> {
    pub fn name() -> &'static str {
        // Use a global cache with lazy initialization
        use std::any::TypeId;
        use std::collections::HashMap;
        use std::sync::Mutex;
        use std::sync::OnceLock;

        static CACHE: OnceLock<Mutex<HashMap<TypeId, &'static str>>> = OnceLock::new();
        let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

        let type_id = TypeId::of::<T>();

        // Try to get from cache first
        {
            let cache_guard = cache.lock().unwrap();
            if let Some(&name) = cache_guard.get(&type_id) {
                return name;
            }
        }

        // Not in cache, create it once and leak it (only once per type)
        let formatted_name = format!("Targeted({})", T::NAME);
        let static_name = Box::leak(formatted_name.into_boxed_str());

        // Store in cache for future use
        {
            let mut cache_guard = cache.lock().unwrap();
            cache_guard.insert(type_id, static_name);
        }

        static_name
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "T: NetworkMessage")]
pub struct PreviousMessage<T: NetworkMessage> {
    // Empty struct - only used for type information
    #[serde(skip)]
    _phantom: std::marker::PhantomData<T>,
    // Add a marker field that will actually be serialized
    #[serde(default)]
    _marker: bool,
}

impl<T: NetworkMessage> PreviousMessage<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            _marker: false,
        }
    }

    pub fn name() -> &'static str {
        // Use a global cache with lazy initialization
        use std::any::TypeId;
        use std::collections::HashMap;
        use std::sync::Mutex;
        use std::sync::OnceLock;

        static CACHE: OnceLock<Mutex<HashMap<TypeId, &'static str>>> = OnceLock::new();
        let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

        let type_id = TypeId::of::<T>();

        // Try to get from cache first
        {
            let cache_guard = cache.lock().unwrap();
            if let Some(&name) = cache_guard.get(&type_id) {
                return name;
            }
        }

        // Not in cache, create it once and leak it (only once per type)
        let formatted_name = format!("PreviousMessage({})", T::NAME);
        let static_name = Box::leak(formatted_name.into_boxed_str());

        // Store in cache for future use
        {
            let mut cache_guard = cache.lock().unwrap();
            cache_guard.insert(type_id, static_name);
        }

        static_name
    }
}

impl<T: NetworkMessage> NetworkMessage for PreviousMessage<T> {
    const NAME: &'static str = "eventwork::PreviousMessage";
}

/// Marks a type as a subscription message that can be used in a pub/sub pattern.
///
/// This trait now works with both explicit `NetworkMessage` types and auto-generated
/// `EventworkMessage` types, providing a unified subscription API.
///
/// # Type Parameters
/// * `Request` - The message type used to initiate a subscription
/// * `Unsubscribe` - The message type used to terminate a subscription
/// * `SubscriptionParams` - Parameters needed to create a subscription request
///
/// # Examples
/// ```rust
/// use eventwork_common::{NetworkMessage, SubscriptionMessage};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug)]
/// struct GameUpdate {
///     game_id: String,
///     state: Vec<u8>,
/// }
///
/// #[derive(Serialize, Deserialize, Debug)]
/// struct SubscribeToGame {
///     game_id: String,
/// }
///
/// #[derive(Serialize, Deserialize, Debug)]
/// struct UnsubscribeFromGame {
///     game_id: String,
/// }
///
/// impl NetworkMessage for GameUpdate {
///     const NAME: &'static str = "game:GameUpdate";
/// }
///
/// impl NetworkMessage for SubscribeToGame {
///     const NAME: &'static str = "game:Subscribe";
/// }
///
/// impl NetworkMessage for UnsubscribeFromGame {
///     const NAME: &'static str = "game:Unsubscribe";
/// }
///
/// impl SubscriptionMessage for GameUpdate {
///     type SubscribeRequest = SubscribeToGame;
///     type UnsubscribeRequest = UnsubscribeFromGame;
///     type SubscriptionParams = String;
///
///     fn get_subscription_params(&self) -> Self::SubscriptionParams {
///         self.game_id.clone()
///     }
///
///     fn create_subscription_request(params: Self::SubscriptionParams) -> Self::SubscribeRequest {
///         SubscribeToGame { game_id: params }
///     }
///
///     fn create_unsubscribe_request(params: Self::SubscriptionParams) -> Self::UnsubscribeRequest {
///         UnsubscribeFromGame { game_id: params }
///     }
/// }
/// ```
///
pub trait SubscriptionMessage: EventworkMessage {
    /// The message type used to request a subscription
    type SubscribeRequest: EventworkMessage
        + Serialize
        + DeserializeOwned
        + Send
        + Sync
        + Debug
        + 'static;

    /// The message type used to terminate a subscription
    type UnsubscribeRequest: EventworkMessage
        + Serialize
        + DeserializeOwned
        + Send
        + Sync
        + Debug
        + 'static;

    /// Parameters needed to create subscription/unsubscribe requests
    type SubscriptionParams: Serialize
        + DeserializeOwned
        + Send
        + Sync
        + Debug
        + PartialEq
        + Clone
        + 'static;

    /// Returns the subscription parameters associated with this message
    /// This allows clients to match incoming messages with their original subscription parameters
    fn get_subscription_params(&self) -> Self::SubscriptionParams;

    /// Creates a subscription request from the given parameters
    fn create_subscription_request(
        subscription_params: Self::SubscriptionParams,
    ) -> Self::SubscribeRequest;

    /// Creates an unsubscribe request from the given parameters
    fn create_unsubscribe_request(
        subscription_params: Self::SubscriptionParams,
    ) -> Self::UnsubscribeRequest;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eventwork_message_caching() {
        #[derive(Serialize, Deserialize)]
        struct TestMessage {
            data: String
        }

        let name1 = TestMessage::type_name();
        let name2 = TestMessage::type_name();

        // Should return same pointer (cached)
        assert_eq!(name1 as *const str, name2 as *const str);
        assert!(name1.contains("TestMessage"));
    }

    #[test]
    fn test_different_types_different_names() {
        #[derive(Serialize, Deserialize)]
        struct TypeA {
            x: i32
        }

        #[derive(Serialize, Deserialize)]
        struct TypeB {
            x: i32
        }

        let name_a = TypeA::type_name();
        let name_b = TypeB::type_name();

        assert_ne!(name_a, name_b);
        assert!(name_a.contains("TypeA"));
        assert!(name_b.contains("TypeB"));
    }

    #[test]
    fn test_generic_types() {
        #[derive(Serialize, Deserialize)]
        struct Generic<T> {
            value: T
        }

        let name_i32 = Generic::<i32>::type_name();
        let name_string = Generic::<String>::type_name();

        assert_ne!(name_i32, name_string);
        assert!(name_i32.contains("Generic"));
        assert!(name_string.contains("Generic"));
    }

    #[test]
    fn test_network_message_kind() {
        #[derive(Serialize, Deserialize)]
        struct ExplicitMsg {
            data: String
        }

        impl NetworkMessage for ExplicitMsg {
            const NAME: &'static str = "test:ExplicitMsg";
        }

        assert_eq!(ExplicitMsg::message_kind(), "test:ExplicitMsg");
    }

    #[test]
    fn test_eventwork_message_type_name() {
        #[derive(Serialize, Deserialize)]
        struct AutoMsg {
            data: String
        }

        // For types without NetworkMessage, use type_name() directly
        let name = AutoMsg::type_name();
        assert!(name.contains("AutoMsg"));
    }

    #[test]
    fn test_any_network_message_with_explicit() {
        #[derive(Serialize, Deserialize)]
        struct ExplicitMessage {
            content: String
        }

        impl NetworkMessage for ExplicitMessage {
            const NAME: &'static str = "explicit:Message";
        }

        // Should use NetworkMessage implementation
        let kind = ExplicitMessage::message_kind();
        assert_eq!(kind, "explicit:Message");
    }
}
