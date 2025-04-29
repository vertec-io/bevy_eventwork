use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
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
/// Marks a type as an eventwork message
pub trait NetworkMessage: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// A unique name to identify your message, this needs to be unique __across all included crates__
    ///
    /// A good combination is crate name + struct name.
    const NAME: &'static str;
}


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
        // Creates a unique name for each TargetedMessage<T> type
        // 1. Memory is only leaked once per message type during registration
        // 2. The string needs to live for the entire program lifetime
        // 3. The leaked memory is automatically freed when the program exits
        // 4. Provides zero-cost lookups compared to String alternatives
        Box::leak(format!("Targeted({})", T::NAME).into_boxed_str())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "T: NetworkMessage")]
pub struct PreviousMessage<T: NetworkMessage> {
    // Empty struct - only used for type information
    #[serde(skip)]
    _phantom: std::marker::PhantomData<T>,
}

impl<T: NetworkMessage> PreviousMessage<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData
        }
    }

    pub fn name() -> &'static str {
        // Creates a unique name for each TargetedMessage<T> type
        // 1. Memory is only leaked once per message type during registration
        // 2. The string needs to live for the entire program lifetime
        // 3. The leaked memory is automatically freed when the program exits
        // 4. Provides zero-cost lookups compared to String alternatives
        Box::leak(format!("PreviousMessage({})", T::NAME).into_boxed_str())
    }
}

impl<T: NetworkMessage> NetworkMessage for PreviousMessage<T> {
    const NAME: &'static str = "eventwork::PreviousMessage";
}

/// Marks a type as a subscription message that can be used in a pub/sub pattern.
///
/// # Type Parameters
/// * `Request` - The message type used to initiate a subscription
/// * `Unsubscribe` - The message type used to terminate a subscription
/// * `SubscriptionParams` - Parameters needed to create a subscription request
///
/// # Examples
/// ```rust
/// use eventwork::{NetworkMessage, SubscriptionMessage};
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
pub trait SubscriptionMessage: NetworkMessage {
    /// The message type used to request a subscription
    type SubscribeRequest: NetworkMessage + Serialize + DeserializeOwned + Send + Sync + Debug + 'static;
    
    /// The message type used to terminate a subscription
    type UnsubscribeRequest: NetworkMessage + Serialize + DeserializeOwned + Send + Sync + Debug + 'static;
    
    /// Parameters needed to create subscription/unsubscribe requests
    type SubscriptionParams: Serialize + DeserializeOwned + Send + Sync + Debug + PartialEq + Clone + 'static;
    
    /// Returns the subscription parameters associated with this message
    /// This allows clients to match incoming messages with their original subscription parameters
    fn get_subscription_params(&self) -> Self::SubscriptionParams;

    /// Creates a subscription request from the given parameters
    fn create_subscription_request(subscription_params: Self::SubscriptionParams) -> Self::SubscribeRequest;

    /// Creates an unsubscribe request from the given parameters
    fn create_unsubscribe_request(subscription_params: Self::SubscriptionParams) -> Self::UnsubscribeRequest;
}
