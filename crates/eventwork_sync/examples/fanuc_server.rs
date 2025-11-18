use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use bevy::tasks::{TaskPool, TaskPoolBuilder};
use eventwork::{EventworkRuntime, Network};
use eventwork_sync::{AppEventworkSyncExt, EventworkSyncPlugin};
use eventwork_websockets::{NetworkSettings, WebSocketProvider};
use fanuc_shared::{RobotPosition, RobotStatus, JointAngles, RobotInfo, JogCommand};

/// FANUC robot simulator server using eventwork_sync.
///
/// Run with:
///   cargo run -p eventwork_sync --example fanuc_server --features runtime
///
/// Then connect a client to ws://127.0.0.1:8082.
fn main() {
    let mut app = App::new();

    app.add_plugins((MinimalPlugins, bevy::log::LogPlugin::default()));

    // Eventwork networking over WebSockets.
    app.add_plugins(eventwork::EventworkPlugin::<WebSocketProvider, TaskPool>::default());
    app.insert_resource(EventworkRuntime(TaskPoolBuilder::new().num_threads(2).build()));
    app.insert_resource(NetworkSettings::default());

    // Install the sync middleware so components can be observed/mutated.
    app.add_plugins(EventworkSyncPlugin::<WebSocketProvider>::default());

    // Register robot components for synchronization.
    app.sync_component::<RobotPosition>(None);
    app.sync_component::<RobotStatus>(None);
    app.sync_component::<JointAngles>(None);
    app.sync_component::<RobotInfo>(None);
    app.sync_component::<JogCommand>(None);

    app.add_systems(Startup, (setup_robot, setup_networking));
    app.add_systems(Update, (process_jog_commands, update_robot_status));

    app.run();
}

fn setup_robot(mut commands: Commands) {
    // Spawn the robot entity with all its components
    commands.spawn((
        Name::new("FANUC_LR_Mate_200iD"),
        RobotInfo {
            name: "FANUC Robot".to_string(),
            model: "LR Mate 200iD".to_string(),
        },
        RobotPosition::default(),
        RobotStatus::default(),
        JointAngles::default(),
    ));

    info!("FANUC robot initialized");
}

fn setup_networking(
    mut net: ResMut<Network<WebSocketProvider>>,
    settings: Res<NetworkSettings>,
    task_pool: Res<EventworkRuntime<TaskPool>>,
) {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8082);

    match net.listen(addr, &task_pool.0, &settings) {
        Ok(_) => info!("FANUC server listening on {addr}"),
        Err(err) => {
            error!("Could not start listening: {err}");
            panic!("Failed to bind WebSocket listener");
        }
    }
}

fn process_jog_commands(
    mut commands: Commands,
    mut robot_query: Query<(Entity, &mut RobotPosition, &mut RobotStatus), Without<JogCommand>>,
    jog_query: Query<(Entity, &JogCommand)>,
) {
    use fanuc_shared::axis::RobotAxis;

    for (jog_entity, jog_cmd) in &jog_query {
        // Find the robot (assuming single robot for now)
        if let Ok((_robot_entity, mut position, mut status)) = robot_query.single_mut() {
            // Apply the jog command to the robot position
            let delta = jog_cmd.direction.sign() * jog_cmd.distance;

            match jog_cmd.axis {
                RobotAxis::X => position.x += delta,
                RobotAxis::Y => position.y += delta,
                RobotAxis::Z => position.z += delta,
                RobotAxis::W => position.w += delta,
                RobotAxis::P => position.p += delta,
                RobotAxis::R => position.r += delta,
            }

            status.in_motion = true;

            info!(
                "Jogged {:?} {:?} by {:.2} - New position: X:{:.2} Y:{:.2} Z:{:.2}",
                jog_cmd.axis, jog_cmd.direction, delta, position.x, position.y, position.z
            );
        }

        // Remove the processed jog command
        commands.entity(jog_entity).despawn();
    }
}

fn update_robot_status(
    time: Res<Time>,
    mut elapsed: Local<f32>,
    mut query: Query<&mut RobotStatus>,
) {
    *elapsed += time.delta_secs();

    // Reset in_motion flag after a short delay
    if *elapsed >= 0.5 {
        *elapsed = 0.0;
        for mut status in &mut query {
            if status.in_motion {
                status.in_motion = false;
            }
        }
    }
}

