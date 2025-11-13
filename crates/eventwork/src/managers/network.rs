use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use async_channel::unbounded;
use bevy::prelude::*;
use dashmap::DashMap;
use futures_lite::StreamExt;
use tracing::{debug, error, trace, warn};

use super::{Network, NetworkProvider};
use crate::{
    AsyncChannel,
    Connection,
    NetworkData,
    NetworkEvent,
    OutboundMessage,
    Runtime,
    // error::NetworkError,
    // network_message::NetworkMessage,
    runtime::{EventworkRuntime, run_async},
};
use eventwork_common::error::NetworkError;
use eventwork_common::{
    ConnectionId, NetworkPacket, SubscriptionMessage, TargetedMessage,
    EventworkMessage,
};
#[cfg(feature = "cache_messages")]
use eventwork_common::PreviousMessage;

impl<NP: NetworkProvider> std::fmt::Debug for Network<NP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Network [{} Connected Clients]",
            self.established_connections.len()
        )
    }
}

impl<NP: NetworkProvider> Network<NP> {
    pub(crate) fn new(_provider: NP) -> Self {
        Self {
            recv_message_map: Arc::new(DashMap::new()),
            recv_message_map_by_hash: Arc::new(DashMap::new()),
            hash_to_typename: Arc::new(DashMap::new()),
            #[cfg(feature = "cache_messages")]
            last_messages: Arc::new(DashMap::new()),
            established_connections: Arc::new(DashMap::new()),
            new_connections: AsyncChannel::new(),
            disconnected_connections: AsyncChannel::new(),
            error_channel: AsyncChannel::new(),
            server_handle: None,
            connection_tasks: Arc::new(DashMap::new()),
            connection_task_counts: AtomicU32::new(0),
            connection_count: 1, // SERVER reserved ID 0
        }
    }

    /// Returns true if there are any active connections
    #[inline(always)]
    pub fn has_connections(&self) -> bool {
        !self.established_connections.is_empty()
    }

    /// Returns true if a specific connection exists
    #[inline(always)]
    pub fn has_connection(&self, conn_id: ConnectionId) -> bool {
        self.established_connections.contains_key(&conn_id)
    }

    /// Check if a message type is registered
    ///
    /// This is primarily useful for testing and debugging.
    pub fn is_message_registered(&self, message_name: &str) -> bool {
        self.recv_message_map.contains_key(message_name)
    }

    /// Get all registered message names
    ///
    /// This is primarily useful for testing and debugging.
    pub fn registered_message_names(&self) -> Vec<String> {
        self.recv_message_map.iter()
            .map(|entry| entry.key().to_string())
            .collect()
    }

    /// Start listening for new clients
    ///
    /// ## Note
    /// If you are already listening for new connections, this will cancel the original listen
    pub fn listen<RT: Runtime>(
        &mut self,
        accept_info: NP::AcceptInfo,
        runtime: &RT,
        network_settings: &NP::NetworkSettings,
    ) -> Result<(), NetworkError> {
        self.stop();

        let new_connections = self.new_connections.sender.clone();
        let error_sender = self.error_channel.sender.clone();
        let settings = network_settings.clone();

        trace!("Started listening");

        self.server_handle = Some(Box::new(run_async(
            async move {
                let accept = NP::accept_loop(accept_info, settings).await;
                match accept {
                    Ok(mut listen_stream) => {
                        while let Some(connection) = listen_stream.next().await {
                            new_connections
                                .send(connection)
                                .await
                                .expect("Connection channel has closed");
                        }
                    }
                    Err(e) => error_sender
                        .send(e)
                        .await
                        .expect("Error channel has closed."),
                }
            },
            runtime,
        )));

        Ok(())
    }

    /// Start async connecting to a remote server.
    pub fn connect<RT: Runtime>(
        &self,
        connect_info: NP::ConnectInfo,
        runtime: &RT,
        network_settings: &NP::NetworkSettings,
    ) {
        debug!("Starting connection");

        let network_error_sender = self.error_channel.sender.clone();
        let connection_event_sender = self.new_connections.sender.clone();
        let settings = network_settings.clone();

        let connection_task_weak = Arc::downgrade(&self.connection_tasks);
        let task_count = self.connection_task_counts.fetch_add(1, Ordering::SeqCst);

        self.connection_tasks.insert(
            task_count,
            Box::new(run_async(
                async move {
                    match NP::connect_task(connect_info, settings).await {
                        Ok(connection) => connection_event_sender
                            .send(connection)
                            .await
                            .expect("Connection channel has closed"),
                        Err(e) => network_error_sender
                            .send(e)
                            .await
                            .expect("Error channel has closed."),
                    };

                    // Remove the connection task from our dictionary of connection tasks
                    connection_task_weak
                        .upgrade()
                        .expect("Network dropped")
                        .remove(&task_count);
                },
                runtime,
            )),
        );
    }

    /// Send a message to a specific client (works for both NetworkMessage and EventworkMessage)
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// // Works with explicit messages
    /// net.send(conn_id, LoginRequest { ... })?;
    ///
    /// // Works with automatic messages
    /// net.send(conn_id, PlayerPosition { ... })?;
    /// ```
    pub fn send<T: EventworkMessage>(
        &self,
        client_id: ConnectionId,
        message: T,
    ) -> Result<(), NetworkError> {
        let connection = match self.established_connections.get(&client_id) {
            Some(conn) => conn,
            None => return Err(NetworkError::ConnectionNotFound(client_id)),
        };

        let packet = NetworkPacket {
            type_name: T::type_name().to_string(),
            schema_hash: T::schema_hash(),
            data: bincode::serialize(&message).map_err(|_| NetworkError::Serialization)?,
        };

        match connection.send_message.try_send(packet) {
            Ok(_) => (),
            Err(err) => {
                error!("There was an error sending a packet: {}", err);
                return Err(NetworkError::ChannelClosed(client_id));
            }
        }

        Ok(())
    }



    /// Broadcast a message to all connected clients (works for both message types)
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// net.broadcast(GameStateUpdate { ... });
    /// ```
    pub fn broadcast<T: EventworkMessage + Clone>(&self, message: T) {
        let serialized_message = bincode::serialize(&message).expect("Couldn't serialize message!");
        for connection in self.established_connections.iter() {
            let packet = NetworkPacket {
                type_name: T::type_name().to_string(),
                schema_hash: T::schema_hash(),
                data: serialized_message.clone(),
            };

            match connection.send_message.try_send(packet) {
                Ok(_) => (),
                Err(err) => {
                    warn!("Could not send to client because: {}", err);
                }
            }
        }
    }

    /// Broadcast a message to all connected clients except the specified one
    ///
    /// This is useful for chat applications where you want to send a message to all
    /// clients except the sender (who already sees their own message optimistically).
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// // Broadcast to everyone except the sender
    /// net.broadcast_except(sender_id, ChatMessage { ... });
    /// ```
    pub fn broadcast_except<T: EventworkMessage + Clone>(&self, except: ConnectionId, message: T) {
        let serialized_message = bincode::serialize(&message).expect("Couldn't serialize message!");
        for connection in self.established_connections.iter() {
            // Skip the excluded connection
            if *connection.key() == except {
                continue;
            }

            let packet = NetworkPacket {
                type_name: T::type_name().to_string(),
                schema_hash: T::schema_hash(),
                data: serialized_message.clone(),
            };

            match connection.send_message.try_send(packet) {
                Ok(_) => (),
                Err(err) => {
                    warn!("Could not send to client because: {}", err);
                }
            }
        }
    }

    /// Disconnect all clients and stop listening for new ones
    ///
    /// ## Notes
    /// This operation is idempotent and will do nothing if you are not actively listening
    pub fn stop(&mut self) {
        if let Some(mut conn) = self.server_handle.take() {
            conn.abort();
            for conn in self.established_connections.iter() {
                match self.disconnected_connections.sender.try_send(*conn.key()) {
                    Ok(_) => (),
                    Err(err) => warn!("Could not send to client because: {}", err),
                }
            }
            self.established_connections.clear();
            self.recv_message_map.clear();
            #[cfg(feature = "cache_messages")]
            self.last_messages.clear();

            while self.new_connections.receiver.try_recv().is_ok() {}
        }
    }

    /// Disconnect a specific client
    pub fn disconnect(&self, conn_id: ConnectionId) -> Result<(), NetworkError> {
        let connection = if let Some(conn) = self.established_connections.remove(&conn_id) {
            conn
        } else {
            return Err(NetworkError::ConnectionNotFound(conn_id));
        };

        connection.1.stop();

        Ok(())
    }
}

pub(crate) fn handle_new_incoming_connections<NP: NetworkProvider, RT: Runtime>(
    mut server: ResMut<Network<NP>>,
    runtime: Res<EventworkRuntime<RT>>,
    network_settings: Res<NP::NetworkSettings>,
    mut network_events: MessageWriter<NetworkEvent>,
) {
    while let Ok(new_conn) = server.new_connections.receiver.try_recv() {
        let id = server.connection_count;
        let conn_id = ConnectionId { id };
        server.connection_count += 1;

        let (read_half, write_half) = NP::split(new_conn);
        let recv_message_map = server.recv_message_map.clone();
        let hash_to_typename = server.hash_to_typename.clone();
        let read_network_settings = network_settings.clone();
        let write_network_settings = network_settings.clone();
        let disconnected_connections = server.disconnected_connections.sender.clone();

        let (outgoing_tx, outgoing_rx) = unbounded();
        let (incoming_tx, incoming_rx) = unbounded();

        server.established_connections.insert(
                conn_id,
                Connection {
                    receive_task: Box::new(run_async(async move {
                        trace!("Starting listen task for {}", id);
                        NP::recv_loop(read_half, incoming_tx, read_network_settings).await;

                        match disconnected_connections.send(conn_id).await {
                            Ok(_) => (),
                            Err(_) => {
                                error!("Could not send disconnected event, because channel is disconnected");
                            }
                        }
                    }, &runtime.0)),
                    map_receive_task: Box::new(run_async(async move{
                        while let Ok(packet) = incoming_rx.recv().await{
                            // Hybrid lookup: try type_name first (fast path), then schema_hash (fallback)
                            if let Some(mut packets) = recv_message_map.get_mut(&packet.type_name[..]) {
                                #[cfg(feature = "debug_messages")]
                                {
                                    println!("Received message '{}' (matched by type_name)", packet.type_name);
                                }
                                packets.push((conn_id, packet.data));
                            } else if let Some(registered_typename) = hash_to_typename.get(&packet.schema_hash) {
                                // Schema hash matched! Get the registered type name and push to that queue
                                let typename_key = *registered_typename.value();
                                drop(registered_typename); // Release the read lock

                                if let Some(mut packets) = recv_message_map.get_mut(typename_key) {
                                    #[cfg(feature = "debug_messages")]
                                    {
                                        println!("Received message '{}' (matched by schema_hash: 0x{:016x}, registered as: {})",
                                                 packet.type_name, packet.schema_hash, typename_key);
                                    }
                                    packets.push((conn_id, packet.data));
                                } else {
                                    error!("Schema hash matched but type_name not found in recv_message_map! Hash: 0x{:016x}, registered typename: {}",
                                           packet.schema_hash, typename_key);
                                }
                            } else {
                                println!("Eventwork could not find a registration for message type: '{}' with schema hash: 0x{:016x}",
                                         packet.type_name, packet.schema_hash);
                                error!("Could not find existing entries for message type: '{}' (hash: 0x{:016x})",
                                       packet.type_name, packet.schema_hash);
                            }
                        }
                    }, &runtime.0)),
                    send_task: Box::new(run_async(async move {
                        trace!("Starting send task for {}", id);
                        NP::send_loop(write_half, outgoing_rx, write_network_settings).await;
                    }, &runtime.0)),
                    send_message: outgoing_tx,
                    //addr: new_conn.addr,
                },
            );

        network_events.write(NetworkEvent::Connected(conn_id));
    }

    while let Ok(disconnected_connection) = server.disconnected_connections.receiver.try_recv() {
        server
            .established_connections
            .remove(&disconnected_connection);
        network_events.write(NetworkEvent::Disconnected(disconnected_connection));
    }
}

// Since we can't use specialization, we'll just use type_name() for all EventworkMessage types
// and have a separate path for explicit NetworkMessage types via listen_for_message
fn register_message_internal<T: EventworkMessage, NP: NetworkProvider>(app: &mut App) -> &mut App {
    let server = app.world_mut().get_resource::<Network<NP>>()
        .expect("Could not find `Network`. Be sure to include the `EventworkPlugin` before registering messages.");

    let message_name = T::type_name();
    let schema_hash = T::schema_hash();
    let short_name = T::short_name();

    debug!("Registered network message: {} (short: {}, hash: 0x{:016x})",
           message_name, short_name, schema_hash);

    // Check for duplicate registration by type_name
    assert!(
        !server.recv_message_map.contains_key(message_name),
        "Duplicate registration of message: {}",
        message_name
    );

    // Check for schema hash collision with different type
    if let Some(existing_typename) = server.hash_to_typename.get(&schema_hash) {
        let existing = *existing_typename.value();
        if existing != message_name {
            panic!(
                "Schema hash collision! Both '{}' and '{}' have the same short name '{}' (hash: 0x{:016x}). \
                 Please rename one of these types to avoid collision.",
                existing, message_name, short_name, schema_hash
            );
        }
    }

    // Register in both maps
    server.recv_message_map.insert(message_name, Vec::new());
    server.recv_message_map_by_hash.insert(schema_hash, Vec::new());
    server.hash_to_typename.insert(schema_hash, message_name);

    app.add_message::<NetworkData<T>>();
    app.add_systems(PreUpdate, register_message::<T, NP>)
}

/// Helper that registers a message if not already registered, using auto-generated type_name
/// Returns true if registration was performed, false if already registered
fn register_auto_if_needed<T: EventworkMessage, NP: NetworkProvider>(app: &mut App) -> bool {
    let already_registered = {
        let server = app.world().get_resource::<Network<NP>>()
            .expect("Could not find `Network`. Be sure to include the `EventworkPlugin` before registering messages.");

        let message_name = T::type_name();
        server.recv_message_map.contains_key(message_name)
    };

    if already_registered {
        return false;
    }

    register_message_internal::<T, NP>(app);
    true
}

/// A utility trait on [`App`] to easily register [`NetworkMessage`]s
pub trait AppNetworkMessage {
    /// Register a network message type using automatic type name generation
    ///
    /// This method uses `std::any::type_name()` to automatically generate a message name.
    /// The name is cached for performance.
    ///
    /// ## Details
    /// This will:
    /// - Add a new event type of [`NetworkData<T>`]
    /// - Register the type for transformation over the wire using automatic naming
    /// - Internal bookkeeping
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// // Automatic message (no impl needed)
    /// #[derive(Serialize, Deserialize)]
    /// struct PlayerPosition { x: f32, y: f32 }
    /// app.register_network_message::<PlayerPosition, TcpProvider>();
    /// ```
    fn register_network_message<T: EventworkMessage, NP: NetworkProvider>(&mut self) -> &mut Self;

    /// Register a network Outgoing message type
    ///
    /// ## Details
    /// This will:
    /// - Add a new event type of [`OutboundMessage<T>`]
    /// - Register the type for sending/broadcasting over the wire
    fn register_outbound_message<T: EventworkMessage + Clone, NP: NetworkProvider, S: SystemSet>(
        &mut self,
        system_set: S,
    ) -> &mut Self;

    /// Register a targeted network message type using automatic type name generation
    ///
    /// This method works with message types that use auto-generated names via `EventworkMessage`.
    ///
    /// ## Details
    /// This will:
    /// - Add a new event type of [`NetworkData<TargetedMessage<T>>`]
    /// - Register the type for transformation over the wire
    fn register_targeted_message<T: EventworkMessage + Clone, NP: NetworkProvider>(
        &mut self,
    ) -> &mut Self;

    /// Register a subscription message type
    ///
    /// This method works with subscription types that use auto-generated names via `EventworkMessage`.
    /// It automatically registers the subscription message, subscribe request, and unsubscribe request
    /// if they haven't been registered yet.
    ///
    /// ## Details
    /// This will:
    /// - Register the subscription request, unsubscribe message, and subscription updates
    /// - Use automatic type name generation for all three message types
    /// - Add the appropriate event types and system registrations
    ///
    /// ## Example
    /// ```rust,ignore
    /// // With auto-generated names (no trait implementations needed!)
    /// #[derive(SubscribeById, Serialize, Deserialize)]
    /// struct GameUpdate { game_id: String }
    ///
    /// app.register_subscription::<GameUpdate, WebSocketProvider>();
    /// ```
    fn register_subscription<T: SubscriptionMessage, NP: NetworkProvider>(&mut self) -> &mut Self
    where
        T::SubscribeRequest: EventworkMessage,
        T::UnsubscribeRequest: EventworkMessage;
}

impl AppNetworkMessage for App {
    fn register_network_message<T: EventworkMessage, NP: NetworkProvider>(&mut self) -> &mut Self {
        // Use type_name() for all EventworkMessage types
        // This works for both NetworkMessage and non-NetworkMessage types
        register_message_internal::<T, NP>(self)
    }

    fn register_outbound_message<T: EventworkMessage + Clone, NP: NetworkProvider, S: SystemSet>(
        &mut self,
        system_set: S,
    ) -> &mut Self {
        let server = self.world_mut().get_resource::<Network<NP>>()
            .expect("Could not find `Network`. Be sure to include the `ServerPlugin` before listening for server messages.");

        let type_name = T::type_name();
        debug!("Registered a new OutboundMessage: {}", type_name);

        if !server.recv_message_map.contains_key(type_name) {
            server.recv_message_map.insert(type_name, Vec::new());
        }

        // Register to listen for PreviousMessage requests
        #[cfg(feature = "cache_messages")]
        {
            let previous_message_name = PreviousMessage::<T>::name();
            if !server.recv_message_map.contains_key(previous_message_name) {
                server
                    .recv_message_map
                    .insert(previous_message_name, Vec::new());
            }
            self.add_message::<NetworkData<PreviousMessage<T>>>();
            self.add_systems(PreUpdate, register_previous_message::<T, NP>);
            self.add_systems(PreUpdate, handle_previous_message_requests::<T, NP>);
        }

        self.add_message::<OutboundMessage<T>>();

        self.add_systems(
            Update,
            relay_outbound_notifications::<T, NP>.in_set(system_set),
        );

        self
    }

    fn register_targeted_message<T: EventworkMessage + Clone, NP: NetworkProvider>(
        &mut self,
    ) -> &mut Self {
        let already_registered = {
            let server = self.world().get_resource::<Network<NP>>()
                .expect("Could not find `Network`. Be sure to include the `EventworkPlugin` before registering targeted messages.");

            let targeted_message_name = TargetedMessage::<T>::name();
            server.recv_message_map.contains_key(targeted_message_name)
        };

        if already_registered {
            return self;
        }

        let server = self.world_mut().get_resource::<Network<NP>>()
            .expect("Could not find `Network`. Be sure to include the `EventworkPlugin` before registering targeted messages.");

        let targeted_message_name = TargetedMessage::<T>::name();
        server
            .recv_message_map
            .insert(targeted_message_name, Vec::new());

        self.add_message::<NetworkData<TargetedMessage<T>>>();
        self.add_systems(PreUpdate, register_targeted_message::<T, NP>);

        self
    }

    /// Register a subscription message type
    ///
    /// This method works with subscription types that use auto-generated names via `EventworkMessage`.
    /// It automatically registers the subscription message, subscribe request, and unsubscribe request
    /// if they haven't been registered yet.
    ///
    /// **Note**: When using the `SubscribeById` macro, the Subscribe/Unsubscribe types will have
    /// explicit `NetworkMessage` implementations. This method will use those explicit names for
    /// Subscribe/Unsubscribe types, but can use auto-generated names for the base subscription type.
    ///
    /// # Type Parameters
    /// * `T` - The subscription message type implementing `SubscriptionMessage`
    /// * `NP` - The network provider type
    ///
    /// # Examples
    /// ```rust,ignore
    /// // With auto-generated names (no explicit NetworkMessage impl needed for base type)
    /// #[derive(SubscribeById, Serialize, Deserialize)]
    /// struct GameUpdate { game_id: String }
    ///
    /// // No need to implement NetworkMessage for GameUpdate!
    /// app.register_subscription::<GameUpdate, WebSocketProvider>();
    /// ```
    fn register_subscription<T: SubscriptionMessage, NP: NetworkProvider>(
        &mut self,
    ) -> &mut Self
    where
        T::SubscribeRequest: EventworkMessage,
        T::UnsubscribeRequest: EventworkMessage,
    {
        // Register all three message types using auto-generated names
        register_auto_if_needed::<T::SubscribeRequest, NP>(self);
        register_auto_if_needed::<T::UnsubscribeRequest, NP>(self);
        register_auto_if_needed::<T, NP>(self);

        self
    }
}

/// System that processes incoming messages for EventworkMessage types
pub(crate) fn register_message<T, NP: NetworkProvider>(
    net_res: ResMut<Network<NP>>,
    mut events: MessageWriter<NetworkData<T>>,
) where
    T: EventworkMessage,
{
    let name = T::type_name();
    let mut messages = match net_res.recv_message_map.get_mut(name) {
        Some(messages) => messages,
        None => return,
    };

    #[cfg(feature = "cache_messages")]
    if let Some((_, newest_message)) = messages.last() {
        net_res.last_messages.insert(name, newest_message.clone());
    }

    events.write_batch(messages.drain(..).filter_map(|(source, msg)| {
        bincode::deserialize::<T>(&msg)
            .ok()
            .map(|inner| NetworkData { source, inner })
    }));
}

/// Relays outbound notifications to the appropriate clients.
///
/// This system reads outbound messages from the `OutboundMessage<T>` event and
/// sends them either to a specific client or broadcasts them to all connected clients
/// using the provided `Network<NP>` resource.
///
/// # Type Parameters
///
/// * `T` - The type of the network message that implements the `NetworkMessage` trait.
/// * `NP` - The type of the network provider that implements the `NetworkProvider` trait.
///
/// # Parameters
///
/// * `outbound_messages` - A `MessageReader` that reads `OutboundMessage<T>` events,
///   which contain the messages to be sent to clients.
/// * `net` - A `Res<Network<NP>>` resource that provides access to the network
///   for sending and broadcasting messages.
///
/// # Behavior
///
/// The function iterates over all outbound messages:
/// - If the message is designated for a specific client (`for_client` is `Some(client)`),
///   it attempts to send the message to that client using `send_message`.
/// - If the message is intended for all clients (`for_client` is `None`), it broadcasts
///   the message using `broadcast`.
pub fn relay_outbound_notifications<T: EventworkMessage + Clone, NP: NetworkProvider>(
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

/// System that handles requests from clients for the most recent message of a specific type.
///
/// When a client sends a `PreviousMessage<T>`, this system will:
/// 1. Look up the most recent serialized message of type `T` in the `recv_message_map`
/// 2. If found, create a `NetworkPacket` using the existing serialized data
/// 3. Send the packet directly to the requesting client through their connection channel
///
/// This allows clients to request the latest state of any message type they're interested in,
/// without requiring the server to deserialize and re-serialize the data.
///
/// # Type Parameters
/// * `T` - The type of the network message being requested
/// * `NP` - The network provider type
///
/// # Arguments
/// * `previous_message_requests` - Event reader for incoming `PreviousMessage<T>` requests
/// * `server` - The network resource containing connection and message information
#[cfg(feature = "cache_messages")]
fn handle_previous_message_requests<T: EventworkMessage + Clone, NP: NetworkProvider>(
    mut previous_message_requests: MessageReader<NetworkData<PreviousMessage<T>>>,
    server: Res<Network<NP>>,
) {
    for request in previous_message_requests.read() {
        // Get the last message from the cache
        let type_name = T::type_name();
        if let Some(last_message) = server.last_messages.get(type_name) {
            let packet = NetworkPacket {
                type_name: String::from(type_name),
                schema_hash: T::schema_hash(),
                data: last_message.clone(),
            };

            if let Some(connection) = server.established_connections.get(&request.source) {
                let _ = connection.send_message.try_send(packet);
            }
        }
    }
}

/// Registers a targeted message type for the network.
pub fn register_targeted_message<T, NP: NetworkProvider>(
    net_res: ResMut<Network<NP>>,
    mut events: MessageWriter<NetworkData<TargetedMessage<T>>>,
) where
    T: EventworkMessage,
{
    let mut messages = match net_res
        .recv_message_map
        .get_mut(TargetedMessage::<T>::name())
    {
        Some(messages) => messages,
        None => return,
    };

    events.write_batch(messages.drain(..).filter_map(|(source, msg)| {
        match bincode::deserialize::<TargetedMessage<T>>(&msg) {
            Ok(inner) => {
                #[cfg(feature = "debug_messages")]
                println!(
                    "Successfully deserialized message for target: {}",
                    inner.target_id
                );
                Some(NetworkData { source, inner })
            }
            Err(_e) => {
                #[cfg(feature = "debug_messages")]
                println!("Failed to deserialize message: {:?}", _e);
                None
            }
        }
    }));

    // events.send_batch(messages.drain(..).filter_map(|(source, msg)| {
    //     bincode::deserialize::<TargetedMessage<T>>(&msg)
    //         .ok()
    //         .map(|inner| NetworkData { source, inner })
    // }));
}

/// System that registers and processes incoming `PreviousMessage<T>` network messages.
///
/// This system is responsible for:
/// 1. Reading `PreviousMessage<T>` messages from the network message map
/// 2. Deserializing them into proper `NetworkData<PreviousMessage<T>>` events
/// 3. Sending these events through Bevy's event system
///
/// It works in conjunction with `handle_previous_message_requests` to implement the
/// previous message request/response functionality.
///
/// # Type Parameters
/// * `T` - The type of the network message being wrapped in `PreviousMessage`
/// * `NP` - The network provider type
///
/// # Arguments
/// * `net_res` - The network resource containing message queues and connection information
/// * `events` - Event writer for sending `NetworkData<PreviousMessage<T>>` events
#[cfg(feature = "cache_messages")]
pub(crate) fn register_previous_message<T, NP: NetworkProvider>(
    net_res: ResMut<Network<NP>>,
    mut events: MessageWriter<NetworkData<PreviousMessage<T>>>,
) where
    T: EventworkMessage,
{
    let name = PreviousMessage::<T>::name();

    // Get a mutable reference to the messages
    let mut messages = match net_res.recv_message_map.get_mut(name) {
        Some(messages) => messages,
        None => return,
    };

    if messages.is_empty() {
        return;
    }

    #[cfg(feature = "debug_messages")]
    println!(
        "Received a request for PreviousMessage of type : {}",
        T::type_name()
    );

    // Drain the message buffer and send events
    events.write_batch(messages.drain(..).filter_map(|(source, msg)| {
        bincode::deserialize::<PreviousMessage<T>>(&msg)
            .ok()
            .map(|inner| NetworkData { source, inner })
    }));
}
