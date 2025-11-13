//! Scheduled message handling plugin for the hybrid server.
//!
//! This module demonstrates the "scheduled" or "decoupled" approach to handling messages
//! in a multi-protocol server. Game logic writes OutboundMessage<T> events, and the built-in
//! relay system handles the actual network broadcasting in a deterministic system set.
//!
//! **Trade-offs:**
//! - ✅ Complete decoupling - game logic has no Network dependencies
//! - ✅ Deterministic - all messages sent at the same point in the frame
//! - ✅ Easy to test - game logic can be tested without network infrastructure
//! - ✅ Flexible - easy to add new protocols without changing game logic
//! - ❌ Slightly more complex - requires understanding of system sets and OutboundMessage

use bevy::prelude::*;
use eventwork::{AppNetworkMessage, Network, NetworkData, NetworkEvent, OutboundMessage};
use eventwork::tcp::TcpProvider;
use eventwork_websockets::WebSocketProvider;

use super::shared_types;

/// Plugin that implements scheduled message handling.
///
/// This plugin uses the built-in `register_outbound_message` method which automatically
/// sets up the relay system for each provider. This is the recommended approach!
pub struct ScheduledMsgPlugin;

impl Plugin for ScheduledMsgPlugin {
    fn build(&self, app: &mut App) {
        // Define system sets for deterministic message handling
        app.configure_sets(Update, (
            GameLogic,
            NetworkRelay.after(GameLogic),
        ));

        // Register outbound messages for BOTH providers
        // This automatically sets up the relay_outbound system for each provider
        app.register_outbound_message::<shared_types::NewChatMessage, TcpProvider, _>(NetworkRelay.clone());
        app.register_outbound_message::<shared_types::NewChatMessage, WebSocketProvider, _>(NetworkRelay.clone());

        // Add connection event handler (not part of the message flow)
        app.add_systems(Update, handle_connection_events);

        // Add game logic system (reads messages, writes OutboundMessage)
        app.add_systems(Update, handle_messages.in_set(GameLogic));
    }
}

/// System set for game logic that processes incoming messages
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
struct GameLogic;

/// System set for network relay that broadcasts outbound messages
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
struct NetworkRelay;

/// Resource to track connections from both protocols
#[derive(Resource, Default)]
pub struct UnifiedConnectionRegistry {
    pub tcp_connections: Vec<eventwork_common::ConnectionId>,
    pub ws_connections: Vec<eventwork_common::ConnectionId>,
}

impl UnifiedConnectionRegistry {
    pub fn add_tcp(&mut self, id: eventwork_common::ConnectionId) {
        if !self.tcp_connections.contains(&id) {
            self.tcp_connections.push(id);
        }
        info!("📡 TCP connection added: {} (Total TCP: {}, WS: {})", id, self.tcp_connections.len(), self.ws_connections.len());
    }

    pub fn add_ws(&mut self, id: eventwork_common::ConnectionId) {
        if !self.ws_connections.contains(&id) {
            self.ws_connections.push(id);
        }
        info!("🌐 WebSocket connection added: {} (Total TCP: {}, WS: {})", id, self.tcp_connections.len(), self.ws_connections.len());
    }

    pub fn remove_tcp(&mut self, id: eventwork_common::ConnectionId) {
        self.tcp_connections.retain(|&x| x != id);
        info!("📡 TCP connection removed: {} (Total TCP: {}, WS: {})", id, self.tcp_connections.len(), self.ws_connections.len());
    }

    pub fn remove_ws(&mut self, id: eventwork_common::ConnectionId) {
        self.ws_connections.retain(|&x| x != id);
        info!("🌐 WebSocket connection removed: {} (Total TCP: {}, WS: {})", id, self.tcp_connections.len(), self.ws_connections.len());
    }
}

/// Unified connection event handler that processes events from BOTH TCP and WebSocket networks.
fn handle_connection_events(
    tcp_net: Res<Network<TcpProvider>>,
    ws_net: Res<Network<WebSocketProvider>>,
    mut network_events: MessageReader<NetworkEvent>,
    mut registry: ResMut<UnifiedConnectionRegistry>,
) {
    for event in network_events.read() {
        match event {
            NetworkEvent::Connected(conn_id) => {
                // Determine which network this connection belongs to
                let is_tcp = tcp_net.has_connection(*conn_id);
                let is_ws = ws_net.has_connection(*conn_id);

                if is_tcp {
                    info!("📡 TCP client connected: {}", conn_id);
                    registry.add_tcp(*conn_id);
                } else if is_ws {
                    info!("🌐 WebSocket client connected: {}", conn_id);
                    registry.add_ws(*conn_id);
                } else {
                    warn!("Connection event for unknown connection: {}", conn_id);
                }
            }
            NetworkEvent::Disconnected(conn_id) => {
                // Check which registry has this connection
                let was_tcp = registry.tcp_connections.contains(conn_id);
                let was_ws = registry.ws_connections.contains(conn_id);

                if was_tcp {
                    info!("📡 TCP client disconnected: {}", conn_id);
                    registry.remove_tcp(*conn_id);
                } else if was_ws {
                    info!("🌐 WebSocket client disconnected: {}", conn_id);
                    registry.remove_ws(*conn_id);
                }
            }
            NetworkEvent::Error(err) => {
                error!("Network error: {}", err);
            }
        }
    }
}

/// Game logic that processes incoming messages and writes OutboundMessage events.
///
/// This demonstrates the scheduled pattern: game logic has ZERO dependencies on Network resources!
/// It simply reads incoming messages and writes outbound messages. The relay system handles the rest.
fn handle_messages(
    mut new_messages: MessageReader<NetworkData<shared_types::UserChatMessage>>,
    mut outbound: MessageWriter<OutboundMessage<shared_types::NewChatMessage>>,
) {
    for message in new_messages.read() {
        let sender_id = message.source();
        let provider = message.provider_name();

        // Determine log emoji based on provider
        let log_emoji = if provider == "TCP" { "📡" } else { "🌐" };

        info!("{} Received {} message from {}: {}", log_emoji, provider, sender_id, message.message);

        // Create the broadcast message with protocol prefix
        let broadcast_message = shared_types::NewChatMessage {
            name: format!("{}-{}", provider, sender_id),
            message: message.message.clone(),
        };

        // Scheduled pattern: Write OutboundMessage - the built-in relay system handles broadcasting!
        // This completely decouples game logic from network infrastructure
        // The relay_outbound system (registered via register_outbound_message) will automatically
        // broadcast this message to all clients on both TCP and WebSocket providers
        outbound.write(OutboundMessage {
            name: "chat".to_string(),
            message: broadcast_message,
            for_client: None,  // None means broadcast to all
        });
    }
}

