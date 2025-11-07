use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use eventwork::{
    AppNetworkMessage, EventworkPlugin, EventworkRuntime, Network,
    NetworkMessage, ConnectionId,
    tcp::{TcpProvider, NetworkSettings},
};
use serde::{Deserialize, Serialize};

// Explicit message with NetworkMessage implementation
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct ExplicitMessage {
    content: String,
}

impl NetworkMessage for ExplicitMessage {
    const NAME: &'static str = "test:ExplicitMessage";
}

// Automatic message (no NetworkMessage impl)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct AutoMessage {
    content: String,
}

// Helper function to create a test app with minimal setup
fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(EventworkPlugin::<TcpProvider, bevy::tasks::TaskPool>::default());
    app.insert_resource(EventworkRuntime(TaskPoolBuilder::new().num_threads(2).build()));
    app.insert_resource(NetworkSettings::default());
    app
}

#[test]
#[allow(deprecated)]
fn test_register_explicit_message() {
    let mut app = create_test_app();

    // For explicit NAME, use listen_for_message
    app.listen_for_message::<ExplicitMessage, TcpProvider>();

    // Verify registration with explicit name
    let net = app.world().get_resource::<Network<TcpProvider>>().unwrap();
    assert!(net.is_message_registered("test:ExplicitMessage"));
}

#[test]
fn test_register_explicit_message_with_auto_name() {
    let mut app = create_test_app();

    // register_network_message uses type_name() even for NetworkMessage types
    app.register_network_message::<ExplicitMessage, TcpProvider>();

    // Verify registration with type name (not explicit NAME)
    let net = app.world().get_resource::<Network<TcpProvider>>().unwrap();
    let names = net.registered_message_names();
    let has_explicit = names.iter().any(|name| name.contains("ExplicitMessage"));
    assert!(has_explicit, "ExplicitMessage should be registered with type name");

    // Should NOT be registered with explicit NAME when using register_network_message
    assert!(!net.is_message_registered("test:ExplicitMessage"));
}

#[test]
fn test_register_auto_message() {
    let mut app = create_test_app();

    // Should work without NetworkMessage impl
    app.register_network_message::<AutoMessage, TcpProvider>();
    
    // Verify registration (name will be full type path)
    let net = app.world().get_resource::<Network<TcpProvider>>().unwrap();
    let names = net.registered_message_names();
    let has_auto_msg = names.iter().any(|name| name.contains("AutoMessage"));
    assert!(has_auto_msg, "AutoMessage should be registered");
}

#[test]
#[allow(deprecated)]
fn test_mixed_registration() {
    let mut app = create_test_app();

    // Mix explicit (with listen_for_message) and automatic (with register_network_message)
    app.listen_for_message::<ExplicitMessage, TcpProvider>();
    app.register_network_message::<AutoMessage, TcpProvider>();

    // Both should be registered
    let net = app.world().get_resource::<Network<TcpProvider>>().unwrap();
    assert!(net.is_message_registered("test:ExplicitMessage"));
    let names = net.registered_message_names();
    assert!(names.iter().any(|name| name.contains("AutoMessage")));
}

#[test]
#[should_panic(expected = "Duplicate registration")]
fn test_duplicate_registration_panics() {
    let mut app = create_test_app();

    app.register_network_message::<AutoMessage, TcpProvider>();
    app.register_network_message::<AutoMessage, TcpProvider>(); // Should panic
}

#[test]
#[allow(deprecated)]
fn test_backward_compatibility() {
    let mut app = create_test_app();

    // Old API should still work (though deprecated)
    app.listen_for_message::<ExplicitMessage, TcpProvider>();
    
    // Verify registration
    let net = app.world().get_resource::<Network<TcpProvider>>().unwrap();
    assert!(net.is_message_registered("test:ExplicitMessage"));
}

#[test]
#[allow(deprecated)]
fn test_send_explicit_message() {
    let mut app = create_test_app();

    // Use listen_for_message for explicit NAME
    app.listen_for_message::<ExplicitMessage, TcpProvider>();

    let net = app.world().get_resource::<Network<TcpProvider>>().unwrap();

    // Test that send method exists and compiles
    // (We can't actually send without a connection, but we can verify the API works)
    let msg = ExplicitMessage { content: "test".to_string() };
    let result = net.send(ConnectionId { id: 999 }, msg);

    // Should fail because connection doesn't exist, but that's expected
    assert!(result.is_err());
}

#[test]
fn test_send_auto_message() {
    let mut app = create_test_app();

    app.register_network_message::<AutoMessage, TcpProvider>();
    
    let net = app.world().get_resource::<Network<TcpProvider>>().unwrap();
    
    // Test that send method works with auto messages
    let msg = AutoMessage { content: "test".to_string() };
    let result = net.send(ConnectionId { id: 999 }, msg);
    
    // Should fail because connection doesn't exist, but that's expected
    assert!(result.is_err());
}

#[test]
#[allow(deprecated)]
fn test_broadcast_explicit_message() {
    let mut app = create_test_app();

    // Use listen_for_message for explicit NAME
    app.listen_for_message::<ExplicitMessage, TcpProvider>();

    let net = app.world().get_resource::<Network<TcpProvider>>().unwrap();

    // Test that broadcast method exists and compiles
    let msg = ExplicitMessage { content: "test".to_string() };
    net.broadcast(msg);

    // No connections, so nothing happens, but API works
}

#[test]
fn test_broadcast_auto_message() {
    let mut app = create_test_app();

    app.register_network_message::<AutoMessage, TcpProvider>();
    
    let net = app.world().get_resource::<Network<TcpProvider>>().unwrap();
    
    // Test that broadcast method works with auto messages
    let msg = AutoMessage { content: "test".to_string() };
    net.broadcast(msg);
    
    // No connections, so nothing happens, but API works
}

#[test]
fn test_external_type_registration() {
    // Test that we can register types from external crates
    // (simulated by using a type without NetworkMessage impl)

    #[derive(Serialize, Deserialize, Clone)]
    struct ExternalType {
        data: Vec<u8>,
    }

    let mut app = create_test_app();

    // This works because EventworkMessage has a blanket impl
    app.register_network_message::<ExternalType, TcpProvider>();
    
    // Verify registration
    let net = app.world().get_resource::<Network<TcpProvider>>().unwrap();
    let names = net.registered_message_names();
    let has_external = names.iter().any(|name| name.contains("ExternalType"));
    assert!(has_external, "ExternalType should be registered");
}

#[test]
fn test_generic_type_registration() {
    #[derive(Serialize, Deserialize, Clone)]
    struct GenericMessage<T> {
        value: T,
    }

    let mut app = create_test_app();

    // Register different instantiations of the generic type
    app.register_network_message::<GenericMessage<i32>, TcpProvider>();
    app.register_network_message::<GenericMessage<String>, TcpProvider>();
    
    // Both should be registered with different names
    let net = app.world().get_resource::<Network<TcpProvider>>().unwrap();
    let names = net.registered_message_names();
    let registrations: Vec<_> = names.iter()
        .filter(|name| name.contains("GenericMessage"))
        .collect();

    assert_eq!(registrations.len(), 2, "Both generic instantiations should be registered");
}

