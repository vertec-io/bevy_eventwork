use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use bevy::tasks::{TaskPool, TaskPoolBuilder};
use eventwork::{EventworkRuntime, Network, NetworkEvent};
use eventwork_sync::{AppEventworkSyncExt, EventworkSyncPlugin};
use eventwork_websockets::{NetworkSettings, WebSocketProvider};
use serde::{Deserialize, Serialize};

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
    // The fully-qualified type paths are used by the DevTools UI:
    //   - "devtools_demo_server::DemoCounter"
    //   - "devtools_demo_server::DemoFlag"
    app.sync_component::<DemoCounter>(None);
    app.sync_component::<DemoFlag>(None);

    app.add_systems(Startup, (setup_world, setup_networking));
    app.add_systems(Update, tick_counters);

    app.run();
}

#[derive(Component, Reflect, Serialize, Deserialize, Debug, Clone)]
#[reflect(Component)]
struct DemoCounter {
    pub value: i32,
}

#[derive(Component, Reflect, Serialize, Deserialize, Debug, Clone)]
#[reflect(Component)]
struct DemoFlag {
    pub label: String,
    pub enabled: bool,
}

fn setup_world(mut commands: Commands) {
    commands.spawn((
        Name::new("Alpha"),
        DemoCounter { value: 0 },
        DemoFlag {
            label: "Alpha".to_string(),
            enabled: true,
        },
    ));

    commands.spawn((
        Name::new("Beta"),
        DemoCounter { value: 10 },
        DemoFlag {
            label: "Beta".to_string(),
            enabled: false,
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

