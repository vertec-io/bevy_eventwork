//! Immediate message handling plugin for the hybrid server.
//!
//! This module demonstrates the "immediate" or "raw" approach to handling messages
//! in a multi-protocol server. Game logic systems directly use Network<T> resources
//! to send messages, providing maximum control and simplicity.
//!
//! **Trade-offs:**
//! - ✅ Simple and direct - no extra abstractions
//! - ✅ Full control over when messages are sent
//! - ❌ Game logic is coupled to Network resources
//! - ❌ Harder to test without network infrastructure
//! - ❌ Less deterministic - messages sent whenever systems run

use bevy::prelude::*;
use eventwork::{Network, NetworkData, NetworkEvent};
use eventwork::tcp::TcpProvider;
use eventwork_websockets::WebSocketProvider;

use super::shared_types;

/// Plugin that implements immediate message handling.
///
/// This plugin adds systems that directly use Network<TcpProvider> and
/// Network<WebSocketProvider> resources to handle messages and broadcast them.
pub struct ImmediateMsgPlugin;

impl Plugin for ImmediateMsgPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            handle_connection_events,
            handle_messages,
        ));
    }
}

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
///
/// This demonstrates the immediate pattern: we directly check Network resources to determine
/// which protocol the connection belongs to.
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

/// Message handler that directly broadcasts using Network resources.
///
/// This demonstrates the immediate pattern: game logic directly uses Network<T> resources
/// to send messages. This provides maximum control but couples game logic to network infrastructure.
fn handle_messages(
    mut new_messages: MessageReader<NetworkData<shared_types::UserChatMessage>>,
    tcp_net: Res<Network<TcpProvider>>,
    ws_net: Res<Network<WebSocketProvider>>,
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

        // Immediate pattern: Directly broadcast to both networks
        // This is simple and direct, but couples game logic to Network resources
        tcp_net.broadcast(broadcast_message.clone());
        ws_net.broadcast(broadcast_message);
    }
}

