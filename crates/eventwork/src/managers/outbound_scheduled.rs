//! Scheduled outbound message handling.
//!
//! This module provides the "scheduled" strategy for handling outbound messages.
//! Messages are collected throughout the frame and then sent in a deterministic
//! system set, ensuring all messages are sent at the same point in the frame.
//!
//! This approach provides:
//! - **Determinism**: All messages sent at the same point in the frame
//! - **Decoupling**: Game logic doesn't need access to Network resources
//! - **Batching**: Can apply deferred changes before sending to ensure world state is synced
//!
//! # Architecture
//!
//! 1. **Early in frame**: Game logic systems write `OutboundMessage<T>` events
//! 2. **Late in frame**: `relay_outbound_scheduled` reads all `OutboundMessage<T>` and broadcasts them
//!
//! # Example
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use eventwork::{AppNetworkMessage, OutboundMessage};
//! use eventwork::tcp::TcpProvider;
//! use eventwork_websockets::WebSocketProvider;
//!
//! #[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
//! struct GameLogic;
//!
//! #[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
//! struct NetworkRelay;
//!
//! fn setup(app: &mut App) {
//!     // Configure system sets
//!     app.configure_sets(Update, (
//!         GameLogic,
//!         NetworkRelay.after(GameLogic),
//!     ));
//!
//!     // Register outbound message - relay happens in NetworkRelay set
//!     app.register_outbound_message::<ChatMessage, TcpProvider, _>(NetworkRelay);
//!     app.register_outbound_message::<ChatMessage, WebSocketProvider, _>(NetworkRelay);
//!
//!     // Game logic runs in GameLogic set
//!     app.add_systems(Update, handle_chat.in_set(GameLogic));
//! }
//!
//! // Game logic system - no network dependencies!
//! fn handle_chat(
//!     mut outbound: MessageWriter<OutboundMessage<ChatMessage>>,
//! ) {
//!     outbound.write(OutboundMessage::new(
//!         "chat".to_string(),
//!         ChatMessage { text: "Hello!".to_string() }
//!     ));
//!     // Message will be sent later in NetworkRelay set
//! }
//! ```

use bevy::prelude::*;
use crate::{EventworkMessage, NetworkProvider, Network, OutboundMessage};
use tracing::error;

/// Relays outbound messages in a scheduled, deterministic manner.
///
/// This system reads outbound messages from the `OutboundMessage<T>` message queue and
/// sends them either to a specific client or broadcasts them to all connected clients
/// using the provided `Network<NP>` resource.
///
/// Unlike the immediate strategy, this system is designed to run in a specific system set
/// (typically late in the frame) to ensure all messages are sent at the same point,
/// providing deterministic behavior.
///
/// # Type Parameters
///
/// * `T` - The type of the network message that implements the `EventworkMessage` trait.
/// * `NP` - The type of the network provider that implements the `NetworkProvider` trait.
///
/// # Parameters
///
/// * `outbound_messages` - A `MessageReader` that reads `OutboundMessage<T>` messages,
///   which contain the messages to be sent to clients.
/// * `net` - A `Res<Network<NP>>` resource that provides access to the network
///   for sending and broadcasting messages.
///
/// # Behavior
///
/// The function iterates over all outbound messages:
/// - If the message is designated for a specific client (`for_client` is `Some(client)`),
///   it attempts to send the message to that client using `send`.
/// - If the message is intended for all clients (`for_client` is `None`), it broadcasts
///   the message using `broadcast`.
///
/// # Determinism
///
/// By running this system in a specific system set (e.g., after game logic but before
/// the end of the frame), you ensure that:
/// - All messages are sent at the same point in the frame
/// - You can apply deferred changes before sending (e.g., `.apply_deferred()`)
/// - Game logic systems don't need direct access to Network resources
pub fn relay_outbound_scheduled<T: EventworkMessage + Clone, NP: NetworkProvider>(
    mut outbound_messages: MessageReader<OutboundMessage<T>>,
    net: Res<Network<NP>>,
) {
    for notification in outbound_messages.read() {
        match &notification.for_client {
            Some(client) => {
                if let Err(e) = net.send(*client, notification.message.clone()) {
                    error!("Failed to send {} to client {}: {:?}", T::type_name(), client.id, e);
                }
            }
            None => {
                net.broadcast(notification.message.clone());
            }
        }
    }
}

