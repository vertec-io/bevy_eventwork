//! Immediate outbound message handling.
//!
//! This module provides the "immediate" strategy for handling outbound messages.
//! When a system writes an `OutboundMessage<T>`, it is immediately broadcast to
//! all connected clients (or sent to a specific client) in the same frame.
//!
//! This approach is simple and direct, but messages may be sent at different points
//! in the frame depending on when systems run.
//!
//! # Example
//!
//! ```rust,ignore
//! use eventwork::{AppNetworkMessage, OutboundMessage};
//! use eventwork::tcp::TcpProvider;
//!
//! // Register the outbound message type
//! app.register_outbound_message::<ChatMessage, TcpProvider, _>(Update);
//!
//! // In any system, write an outbound message
//! fn my_system(mut outbound: MessageWriter<OutboundMessage<ChatMessage>>) {
//!     outbound.write(OutboundMessage::new(
//!         "chat".to_string(),
//!         ChatMessage { text: "Hello!".to_string() }
//!     ));
//!     // Message is broadcast immediately in the Update schedule
//! }
//! ```

use bevy::prelude::*;
use crate::{EventworkMessage, NetworkProvider, Network, OutboundMessage};
use tracing::error;

/// Relays outbound messages immediately to the appropriate clients.
///
/// This system reads outbound messages from the `OutboundMessage<T>` message queue and
/// sends them either to a specific client or broadcasts them to all connected clients
/// using the provided `Network<NP>` resource.
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
/// Messages are sent immediately when this system runs, which means the timing depends
/// on the system set this is scheduled in.
pub fn relay_outbound_immediate<T: EventworkMessage + Clone, NP: NetworkProvider>(
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

