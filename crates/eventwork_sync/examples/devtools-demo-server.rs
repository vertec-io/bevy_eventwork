use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use bevy::tasks::{TaskPool, TaskPoolBuilder};
use eventwork::{EventworkRuntime, Network, NetworkEvent};
use eventwork_sync::{AppEventworkSyncExt, EventworkSyncPlugin};
use eventwork_websockets::{NetworkSettings, WebSocketProvider};
use demo_shared::{DemoCounter, DemoFlag, ParentEntity, ChildEntities};

/// Simple ECS server used by the devtools demo client.
///
/// Run with:
///   cargo run -p eventwork_sync --example devtools-demo-server
///
/// Then point the devtools demo client at ws://127.0.0.1:8081.
fn main() {
    let mut app = App::new();

    app.add_plugins((MinimalPlugins, bevy::log::LogPlugin::default()));

    // Eventwork networking over WebSockets.
    app.add_plugins(eventwork::EventworkPlugin::<WebSocketProvider, TaskPool>::default());
    app.insert_resource(EventworkRuntime(TaskPoolBuilder::new().num_threads(2).build()));
    app.insert_resource(NetworkSettings::default());

    // Install the sync middleware so components can be observed/mutated.
    app.add_plugins(EventworkSyncPlugin::<WebSocketProvider>::default());

    // Register demo components for synchronization.
    app.sync_component::<DemoCounter>(None);
    app.sync_component::<DemoFlag>(None);

    // Register serializable hierarchy components
    app.sync_component::<ParentEntity>(None);
    app.sync_component::<ChildEntities>(None);

    app.add_systems(Startup, (setup_world, setup_networking));
    app.add_systems(Update, (tick_counters, sync_hierarchy));

    app.run();
}

fn setup_world(mut commands: Commands) {
    // Create a root entity with children to demonstrate hierarchy
    let parent = commands.spawn((
        Name::new("RootEntity"),
        DemoCounter { value: 0 },
        DemoFlag {
            label: "Root".to_string(),
            enabled: true,
        },
    )).id();

    // Create child entities
    let child1 = commands.spawn((
        Name::new("Child1"),
        DemoCounter { value: 10 },
        DemoFlag {
            label: "First Child".to_string(),
            enabled: true,
        },
    )).id();

    let child2 = commands.spawn((
        Name::new("Child2"),
        DemoCounter { value: 20 },
        DemoFlag {
            label: "Second Child".to_string(),
            enabled: false,
        },
    )).id();

    // Create a grandchild
    let grandchild = commands.spawn((
        Name::new("Grandchild"),
        DemoCounter { value: 30 },
        DemoFlag {
            label: "Grandchild".to_string(),
            enabled: true,
        },
    )).id();

    // Set up the hierarchy
    commands.entity(parent).add_children(&[child1, child2]);
    commands.entity(child1).add_children(&[grandchild]);

    // Also create a standalone entity (no parent)
    commands.spawn((
        Name::new("Standalone"),
        DemoCounter { value: 100 },
        DemoFlag {
            label: "Standalone".to_string(),
            enabled: true,
        },
    ));
}

fn setup_networking(
    mut net: ResMut<Network<WebSocketProvider>>,
    settings: Res<NetworkSettings>,
    task_pool: Res<EventworkRuntime<TaskPool>>,
) {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);

    match net.listen(addr, &task_pool.0, &settings) {
        Ok(_) => info!("Devtools demo server listening on {addr}"),
        Err(err) => {
            error!("Could not start listening: {err}");
            panic!("Failed to bind WebSocket listener");
        }
    }
}

fn log_connections(mut events: MessageReader<NetworkEvent>) {
    for event in events.read() {
        match event {
            NetworkEvent::Connected(id) => {
                info!("Client connected: {:?}", id);
            }
            NetworkEvent::Disconnected(id) => {
                info!("Client disconnected: {:?}", id);
            }
            NetworkEvent::Error(err) => {
                error!("Network error: {err}");
            }
        }
    }
}

fn tick_counters(time: Res<Time>, mut elapsed: Local<f32>, mut query: Query<&mut DemoCounter>) {
    *elapsed += time.delta_secs();

    if *elapsed >= 1.0 {
        *elapsed = 0.0;
        for mut counter in &mut query {
            counter.value += 1;
        }
    }
}

/// System that syncs Bevy's ChildOf/Children components to our serializable versions.
/// This allows the devtools to display the entity hierarchy.
fn sync_hierarchy(
    mut commands: Commands,
    // Query entities with ChildOf component
    child_query: Query<(Entity, &ChildOf), Changed<ChildOf>>,
    // Query entities with Children component
    parent_query: Query<(Entity, &Children), Changed<Children>>,
) {
    // Sync ChildOf -> ParentEntity
    for (entity, child_of) in &child_query {
        commands.entity(entity).insert(ParentEntity {
            parent_bits: child_of.parent().to_bits(),
        });
    }

    // Sync Children -> ChildEntities
    for (entity, children) in &parent_query {
        commands.entity(entity).insert(ChildEntities {
            children_bits: children.iter().map(|e| e.to_bits()).collect(),
        });
    }
}
