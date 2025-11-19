//! Basic example server for eventwork_client
//!
//! This example demonstrates a simple Bevy server that:
//! - Spawns entities with Position, Velocity, and EntityName components
//! - Moves entities based on their velocity
//! - Broadcasts component changes to connected clients via eventwork_sync
//!
//! Run with: cargo run -p eventwork_client --example basic_server

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use bevy::tasks::{TaskPool, TaskPoolBuilder};
use eventwork::{EventworkRuntime, Network};
use eventwork_sync::{AppEventworkSyncExt, EventworkSyncPlugin};
use eventwork_websockets::{NetworkSettings, WebSocketProvider};

use eventwork_client_example_shared::{EntityName, Position, Velocity};

fn main() {
    let mut app = App::new();

    app.add_plugins((MinimalPlugins, bevy::log::LogPlugin::default()));

    // Eventwork networking over WebSockets
    app.add_plugins(eventwork::EventworkPlugin::<WebSocketProvider, TaskPool>::default());
    app.insert_resource(EventworkRuntime(TaskPoolBuilder::new().num_threads(2).build()));
    app.insert_resource(NetworkSettings::default());

    // Install the sync middleware
    app.add_plugins(EventworkSyncPlugin::<WebSocketProvider>::default());

    // Register components for synchronization
    app.sync_component::<Position>(None);
    app.sync_component::<Velocity>(None);
    app.sync_component::<EntityName>(None);

    app.add_systems(Startup, (setup, setup_networking));
    app.add_systems(Update, move_entities);

    app.run();
}

fn setup(mut commands: Commands) {
    info!("Starting basic eventwork_client example server");

    // Spawn some entities with position, velocity, and names
    commands.spawn((
        Position { x: 0.0, y: 0.0 },
        Velocity { x: 1.0, y: 0.5 },
        EntityName {
            name: "Entity A".to_string(),
        },
    ));

    commands.spawn((
        Position { x: 100.0, y: 50.0 },
        Velocity { x: -0.5, y: 1.0 },
        EntityName {
            name: "Entity B".to_string(),
        },
    ));

    commands.spawn((
        Position { x: -50.0, y: 100.0 },
        Velocity { x: 0.3, y: -0.8 },
        EntityName {
            name: "Entity C".to_string(),
        },
    ));
}

fn setup_networking(
    mut net: ResMut<Network<WebSocketProvider>>,
    settings: Res<NetworkSettings>,
    task_pool: Res<EventworkRuntime<TaskPool>>,
) {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000);

    match net.listen(addr, &task_pool.0, &settings) {
        Ok(_) => info!("Server listening on ws://{addr}/sync"),
        Err(err) => {
            error!("Could not start listening: {err}");
            panic!("Failed to bind WebSocket listener");
        }
    }
}

fn move_entities(mut query: Query<(&mut Position, &Velocity)>, time: Res<Time>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.x * time.delta_secs() * 10.0;
        pos.y += vel.y * time.delta_secs() * 10.0;

        // Wrap around screen bounds
        if pos.x > 200.0 {
            pos.x = -200.0;
        }
        if pos.x < -200.0 {
            pos.x = 200.0;
        }
        if pos.y > 200.0 {
            pos.y = -200.0;
        }
        if pos.y < -200.0 {
            pos.y = 200.0;
        }
    }
}

