use bevy::tasks::TaskPool;
use bevy::{prelude::*, tasks::TaskPoolBuilder};
use eventwork::{AppNetworkMessage, ConnectionId, EventworkRuntime, Network, NetworkData, NetworkEvent, OutboundMessage};
use eventwork::tcp::{NetworkSettings as TcpNetworkSettings, TcpProvider};
use eventwork_websockets::{NetworkSettings as WsNetworkSettings, WebSocketProvider};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

mod shared_types;

fn main() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::log::LogPlugin::default()));

    // Add BOTH EventworkPlugins - one for TCP, one for WebSocket
    // These create separate Network<TcpProvider> and Network<WebSocketProvider> resources
    app.add_plugins(eventwork::EventworkPlugin::<
        TcpProvider,
        bevy::tasks::TaskPool,
    >::default());
    
    app.add_plugins(eventwork::EventworkPlugin::<
        WebSocketProvider,
        bevy::tasks::TaskPool,
    >::default());

    // Shared runtime for both
    app.insert_resource(EventworkRuntime(
        TaskPoolBuilder::new().num_threads(4).build(),
    ));

    // Insert settings for both providers
    app.insert_resource(TcpNetworkSettings::default());
    app.insert_resource(WsNetworkSettings::default());

    // Define system sets for deterministic message handling
    #[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
    struct GameLogic;

    #[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
    struct NetworkRelay;

    // Configure system set ordering: GameLogic runs first, then NetworkRelay
    // This ensures all game logic completes before messages are sent over the network
    app.configure_sets(Update, (
        GameLogic,
        NetworkRelay.after(GameLogic),
    ));

    // Register incoming messages (what clients send to server)
    app.register_network_message::<shared_types::UserChatMessage, TcpProvider>();
    app.register_network_message::<shared_types::UserChatMessage, WebSocketProvider>();

    // Register outbound message type (what server sends to clients)
    // We add the message type but DON'T use register_outbound_message because we need
    // a custom relay system that handles BOTH TCP and WebSocket providers
    app.add_message::<OutboundMessage<shared_types::NewChatMessage>>();

    // Add our custom hybrid relay system that broadcasts to both providers
    app.add_systems(Update, relay_hybrid_outbound.in_set(NetworkRelay));

    // Unified connection registry
    app.init_resource::<UnifiedConnectionRegistry>();

    app.add_systems(Startup, setup_networking);
    app.add_systems(
        Update,
        (
            handle_connection_events,  // Single unified connection event handler
            handle_messages.in_set(GameLogic),  // Game logic runs first, writes OutboundMessages
        ),
    );

    app.run();
}

/// Tracks all connections from both TCP and WebSocket
#[derive(Resource, Default)]
struct UnifiedConnectionRegistry {
    tcp_connections: Vec<ConnectionId>,
    ws_connections: Vec<ConnectionId>,
}

impl UnifiedConnectionRegistry {
    fn add_tcp(&mut self, id: ConnectionId) {
        self.tcp_connections.push(id);
        info!("📡 TCP connection added: {} (Total TCP: {}, WS: {})", 
            id, self.tcp_connections.len(), self.ws_connections.len());
    }

    fn add_ws(&mut self, id: ConnectionId) {
        self.ws_connections.push(id);
        info!("🌐 WebSocket connection added: {} (Total TCP: {}, WS: {})", 
            id, self.tcp_connections.len(), self.ws_connections.len());
    }

    fn remove_tcp(&mut self, id: ConnectionId) {
        self.tcp_connections.retain(|&conn_id| conn_id != id);
        info!("📡 TCP connection removed: {} (Total TCP: {}, WS: {})", 
            id, self.tcp_connections.len(), self.ws_connections.len());
    }

    fn remove_ws(&mut self, id: ConnectionId) {
        self.ws_connections.retain(|&conn_id| conn_id != id);
        info!("🌐 WebSocket connection removed: {} (Total TCP: {}, WS: {})", 
            id, self.tcp_connections.len(), self.ws_connections.len());
    }
}

fn setup_networking(
    mut tcp_net: ResMut<Network<TcpProvider>>,
    mut ws_net: ResMut<Network<WebSocketProvider>>,
    tcp_settings: Res<TcpNetworkSettings>,
    ws_settings: Res<WsNetworkSettings>,
    task_pool: Res<EventworkRuntime<TaskPool>>,
) {
    // Start TCP server on port 3030
    match tcp_net.listen(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3030),
        &task_pool.0,
        &tcp_settings,
    ) {
        Ok(_) => info!("📡 TCP server listening on 127.0.0.1:3030"),
        Err(err) => {
            error!("Could not start TCP server: {}", err);
            panic!();
        }
    }

    // Start WebSocket server on port 8081
    match ws_net.listen(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
        &task_pool.0,
        &ws_settings,
    ) {
        Ok(_) => info!("🌐 WebSocket server listening on 127.0.0.1:8081"),
        Err(err) => {
            error!("Could not start WebSocket server: {}", err);
            panic!();
        }
    }

    info!("🚀 Hybrid server started! Accepting both TCP and WebSocket connections.");
}

/// Unified connection event handler that processes events from BOTH TCP and WebSocket networks
/// This is necessary because both Network<TcpProvider> and Network<WebSocketProvider>
/// write to the same global MessageWriter<NetworkEvent>, so we need a single handler
/// that determines which protocol the event came from.
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

/// Unified message handler that processes messages from BOTH TCP and WebSocket clients.
///
/// This demonstrates the "scheduled outbound message" pattern:
/// 1. Game logic (this function) runs in the GameLogic system set
/// 2. It writes OutboundMessage<T> events instead of calling net.send() directly
/// 3. The relay system runs later in the NetworkRelay system set and broadcasts the messages
///
/// Benefits:
/// - Complete decoupling: No Network resources needed at all!
/// - Determinism: All messages are sent at the same point in the frame
/// - Simplicity: Just read incoming messages and write outbound messages
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

        // Write a single OutboundMessage - the relay system will handle broadcasting
        // to all clients on all providers
        outbound.write(OutboundMessage {
            name: "chat".to_string(),
            message: broadcast_message,
            for_client: None,  // None means broadcast to all
        });
    }
}

/// Custom relay system for hybrid server that broadcasts OutboundMessages to BOTH providers.
///
/// This demonstrates the scheduled outbound message pattern for a multi-provider setup:
/// - Game logic writes OutboundMessage<T> events
/// - This relay system runs in the NetworkRelay set and broadcasts to all providers
/// - Ensures deterministic message sending across all protocols
fn relay_hybrid_outbound(
    mut outbound_messages: MessageReader<OutboundMessage<shared_types::NewChatMessage>>,
    tcp_net: Res<Network<TcpProvider>>,
    ws_net: Res<Network<WebSocketProvider>>,
) {
    for notification in outbound_messages.read() {
        match &notification.for_client {
            Some(client) => {
                // Send to specific client - try both providers
                if tcp_net.send(*client, notification.message.clone()).is_err() {
                    // If TCP fails, try WebSocket
                    if let Err(e) = ws_net.send(*client, notification.message.clone()) {
                        warn!("Failed to send to client {} via both providers: {}", client, e);
                    }
                }
            }
            None => {
                // Broadcast to all clients on both providers
                tcp_net.broadcast(notification.message.clone());
                ws_net.broadcast(notification.message.clone());
            }
        }
    }
}

