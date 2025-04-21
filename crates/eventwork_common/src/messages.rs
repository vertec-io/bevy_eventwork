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