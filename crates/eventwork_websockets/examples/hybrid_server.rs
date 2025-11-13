use bevy::tasks::TaskPool;
use bevy::{prelude::*, tasks::TaskPoolBuilder};
use eventwork::{AppNetworkMessage, ConnectionId, EventworkRuntime, Network, NetworkData, NetworkEvent};
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

    // Register messages for BOTH providers
    // TCP messages
    app.register_network_message::<shared_types::UserChatMessage, TcpProvider>();
    app.register_network_message::<shared_types::NewChatMessage, TcpProvider>();
    app.register_network_message::<shared_types::OutboundTestMessage, TcpProvider>();

    // WebSocket messages
    app.register_network_message::<shared_types::UserChatMessage, WebSocketProvider>();
    app.register_network_message::<shared_types::NewChatMessage, WebSocketProvider>();
    app.register_network_message::<shared_types::OutboundTestMessage, WebSocketProvider>();

    // Unified connection registry
    app.init_resource::<UnifiedConnectionRegistry>();

    app.add_systems(Startup, setup_networking);
    app.add_systems(
        Update,
        (
            handle_connection_events,  // Single unified connection event handler
            handle_messages,           // Single unified message handler
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

/// Unified message handler that processes messages from BOTH TCP and WebSocket clients
/// This is necessary because both Network<TcpProvider> and Network<WebSocketProvider>
/// write to the same global MessageWriter<NetworkData<T>>, so we need a single handler
/// that determines which protocol the message came from and routes it appropriately.
fn handle_messages(
    mut new_messages: MessageReader<NetworkData<shared_types::UserChatMessage>>,
    tcp_net: Res<Network<TcpProvider>>,
    ws_net: Res<Network<WebSocketProvider>>,
    registry: Res<UnifiedConnectionRegistry>,
) {
    for message in new_messages.read() {
        let sender_id = message.source();

        // Determine which protocol this message came from by checking the actual Network resources
        // NOTE: We MUST check the Network resources, not the registry, because connection IDs
        // can overlap between TCP and WebSocket (both can have ID=1, ID=2, etc.)
        let is_tcp = tcp_net.has_connection(*sender_id);
        let is_ws = ws_net.has_connection(*sender_id);

        if !is_tcp && !is_ws {
            warn!("Received message from unknown connection: {}", sender_id);
            continue;
        }

        // Create the broadcast message with appropriate prefix
        let (prefix, log_emoji) = if is_tcp {
            ("TCP", "📡")
        } else {
            ("WS", "🌐")
        };

        info!("{} Received {} message from {}: {}", log_emoji, prefix, sender_id, message.message);

        let broadcast = shared_types::NewChatMessage {
            name: format!("{}-{}", prefix, sender_id),
            message: message.message.clone(),
        };

        // Broadcast to all TCP clients EXCEPT the sender (if sender is TCP)
        for &conn_id in &registry.tcp_connections {
            if !is_tcp || conn_id != *sender_id {
                if let Err(e) = tcp_net.send(conn_id, broadcast.clone()) {
                    warn!("Failed to send to TCP client {}: {}", conn_id, e);
                }
            }
        }

        // Broadcast to all WebSocket clients EXCEPT the sender (if sender is WS)
        for &conn_id in &registry.ws_connections {
            if !is_ws || conn_id != *sender_id {
                if let Err(e) = ws_net.send(conn_id, broadcast.clone()) {
                    warn!("Failed to send to WebSocket client {}: {}", conn_id, e);
                }
            }
        }
    }
}

