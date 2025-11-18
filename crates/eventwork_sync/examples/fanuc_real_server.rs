use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use bevy::prelude::*;
use bevy::tasks::{TaskPool, TaskPoolBuilder};
use bevy_tokio_tasks::{TokioTasksPlugin, TokioTasksRuntime};
use eventwork::{EventworkRuntime, Network};
use eventwork_sync::{AppEventworkSyncExt, EventworkSyncPlugin};
use eventwork_websockets::{NetworkSettings, WebSocketProvider};
use fanuc_real_shared::{RobotPosition, RobotStatus, JointAngles, RobotInfo, MotionCommand};
use fanuc_rmi::{
    drivers::{FanucDriver, FanucDriverConfig},
    dto,
    packets::PacketPriority,
};
use tokio::sync::broadcast;

/// FANUC robot server using real FANUC_RMI_API driver and simulator.
///
/// Prerequisites:
///   1. Start the FANUC simulator:
///      cd ~/dev/Fanuc_RMI_API && cargo run -p sim -- --realtime
///   2. Run this server:
///      cargo run -p eventwork_sync --example fanuc_real_server --features runtime
///   3. Connect a client to ws://127.0.0.1:8082
fn main() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        bevy::log::LogPlugin::default(),
        TokioTasksPlugin::default(),
    ));

    // Eventwork networking over WebSockets
    app.add_plugins(eventwork::EventworkPlugin::<WebSocketProvider, TaskPool>::default());
    app.insert_resource(EventworkRuntime(TaskPoolBuilder::new().num_threads(2).build()));
    app.insert_resource(NetworkSettings::default());

    // Install the sync middleware
    app.add_plugins(EventworkSyncPlugin::<WebSocketProvider>::default());

    // Register robot components for synchronization
    app.sync_component::<RobotPosition>(None);
    app.sync_component::<RobotStatus>(None);
    app.sync_component::<JointAngles>(None);
    app.sync_component::<RobotInfo>(None);
    app.sync_component::<MotionCommand>(None);

    app.add_systems(Startup, (setup_robot, setup_networking, setup_driver));
    app.add_systems(Update, (process_motion_commands, update_robot_state, poll_robot_status));

    app.run();
}

/// Resource holding the FANUC driver wrapped in Arc for sharing with async tasks
#[derive(Resource)]
struct DriverHandle(Arc<FanucDriver>);

/// Resource for receiving responses from the driver
#[derive(Resource)]
struct DriverResponseReceiver(broadcast::Receiver<dto::ResponsePacket>);

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

    info!("FANUC robot entity initialized");
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

fn setup_driver(runtime: ResMut<TokioTasksRuntime>) {
    info!("Connecting to FANUC simulator...");

    let driver_config = FanucDriverConfig {
        addr: "127.0.0.1".to_string(),
        port: 16001,
        max_messages: 30,
    };

    // Spawn async task to connect to the driver
    runtime.spawn_background_task(|mut ctx| async move {
        match FanucDriver::connect(driver_config).await {
            Ok(driver) => {
                info!("✓ Connected to FANUC simulator");

                // Initialize the driver
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                driver.initialize();
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                let driver = Arc::new(driver);

                // Create a channel to convert protocol responses to DTO responses
                let (dto_tx, dto_rx) = broadcast::channel::<dto::ResponsePacket>(100);
                let mut protocol_rx = driver.response_tx.subscribe();

                // Spawn task to convert protocol responses to DTO
                tokio::spawn(async move {
                    while let Ok(protocol_response) = protocol_rx.recv().await {
                        let dto_response: dto::ResponsePacket = protocol_response.into();
                        let _ = dto_tx.send(dto_response);
                    }
                });

                // Send resources back to Bevy
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world.insert_resource(DriverHandle(driver));
                    ctx.world.insert_resource(DriverResponseReceiver(dto_rx));
                    info!("Driver resources inserted into Bevy world");
                }).await;
            }
            Err(e) => {
                error!("✗ Failed to connect to FANUC simulator: {}", e);
                error!("  Make sure the simulator is running:");
                error!("  cd ~/dev/Fanuc_RMI_API && cargo run -p sim -- --realtime");
            }
        }
    });
}

fn process_motion_commands(
    mut commands: Commands,
    driver: Option<Res<DriverHandle>>,
    motion_query: Query<(Entity, &MotionCommand)>,
) {
    let Some(driver) = driver else {
        return; // Driver not yet connected
    };

    for (entity, motion_cmd) in &motion_query {
        // Convert MotionCommand to SendPacket
        let send_packet: fanuc_rmi::packets::SendPacket =
            dto::SendPacket::Instruction(motion_cmd.instruction.clone()).into();

        match driver.0.send_command(send_packet, PacketPriority::Standard) {

            Ok(seq) => {
                info!("Sent motion command with sequence {}", seq);
            }
            Err(e) => {
                error!("Failed to send motion command: {}", e);
            }
        }

        // Remove the processed command
        commands.entity(entity).despawn();
    }
}

fn update_robot_state(
    mut robot_query: Query<(&mut RobotPosition, &mut RobotStatus, &mut JointAngles)>,
    mut response_rx: Option<ResMut<DriverResponseReceiver>>,
) {
    let Some(ref mut response_rx) = response_rx else {
        return; // Driver not yet connected
    };

    // Process all available responses
    while let Ok(response) = response_rx.0.try_recv() {
        match response {
            dto::ResponsePacket::CommandResponse(dto::CommandResponse::FrcReadCartesianPosition(pos_response)) => {
                // Update robot position
                if let Ok((mut position, _, _)) = robot_query.single_mut() {
                    position.0 = pos_response.pos;
                    debug!("Updated position: X:{:.2} Y:{:.2} Z:{:.2}",
                        position.x, position.y, position.z);
                }
            }
            dto::ResponsePacket::CommandResponse(dto::CommandResponse::FrcGetStatus(status_response)) => {
                // Update robot status
                if let Ok((_, mut status, _)) = robot_query.single_mut() {
                    status.servo_ready = status_response.servo_ready != 0;
                    status.tp_enabled = status_response.tp_mode != 0;
                    status.in_motion = status_response.rmi_motion_status != 0;
                    status.error_message = if status_response.error_id != 0 {
                        Some(format!("Error ID: {}", status_response.error_id))
                    } else {
                        None
                    };
                    debug!("Updated status: servo_ready={}, in_motion={}",
                        status.servo_ready, status.in_motion);
                }
            }
            dto::ResponsePacket::CommandResponse(dto::CommandResponse::FrcReadJointAngles(joint_response)) => {
                // Update joint angles
                if let Ok((_, _, mut joints)) = robot_query.single_mut() {
                    joints.0 = joint_response.joint_angles;
                    debug!("Updated joint angles");
                }
            }
            _ => {
                // Other response types - log for debugging
                debug!("Received response: {:?}", response);
            }
        }
    }
}

fn poll_robot_status(
    time: Res<Time>,
    mut elapsed: Local<f32>,
    driver: Option<Res<DriverHandle>>,
) {
    let Some(driver) = driver else {
        return; // Driver not yet connected
    };

    *elapsed += time.delta_secs();

    // Poll status every 100ms
    if *elapsed >= 0.1 {
        *elapsed = 0.0;

        // Request current position
        let pos_packet: fanuc_rmi::packets::SendPacket =
            dto::SendPacket::Command(dto::Command::FrcReadCartesianPosition(
                dto::FrcReadCartesianPosition { group: 1 }
            )).into();
        let _ = driver.0.send_command(pos_packet, PacketPriority::Low);

        // Request current status
        let status_packet: fanuc_rmi::packets::SendPacket =
            dto::SendPacket::Command(dto::Command::FrcGetStatus).into();
        let _ = driver.0.send_command(status_packet, PacketPriority::Low);
    }
}

