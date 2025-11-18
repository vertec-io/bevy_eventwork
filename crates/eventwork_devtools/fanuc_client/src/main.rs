use eventwork_devtools::DevTools;
use eventwork_sync::{
    client_registry::ComponentTypeRegistry,
    client_sync::SyncClient,
    SerializableEntity,
    SyncClientMessage,
    SyncServerMessage,
    SyncItem,
    SubscriptionRequest,
};
use eventwork_common::{codec::EventworkBincodeCodec, NetworkPacket};
use leptos::prelude::*;
use leptos_use::{use_websocket_with_options, DummyEncoder, UseWebSocketOptions, UseWebSocketReturn, core::ConnectionReadyState};
use reactive_graph::traits::Get;
use serde_json::to_value;
use std::sync::Arc;

// Import shared component types
use fanuc_shared::{RobotPosition, RobotStatus, JointAngles, RobotInfo, JogCommand};
use fanuc_shared::axis::RobotAxis;

fn main() {
    console_error_panic_hook::set_once();
    _ = console_log::init_with_level(log::Level::Debug);

    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (host, set_host) = signal("127.0.0.1".to_string());
    let (port, set_port) = signal("8082".to_string());
    let (ws_url, set_ws_url) = signal(None::<&'static str>);

    // Create type registry and register component types
    let mut registry = ComponentTypeRegistry::new();
    registry.register::<RobotPosition>();
    registry.register::<RobotStatus>();
    registry.register::<JointAngles>();
    registry.register::<RobotInfo>();
    registry.register::<JogCommand>();

    let on_connect = move |_| {
        let url_owned = format!("ws://{}:{}", host.get(), port.get());
        let ws_url_static: &'static str = Box::leak(url_owned.into_boxed_str());
        set_ws_url.set(Some(ws_url_static));
    };

    view! {
        <div class="min-h-screen w-screen bg-slate-950 text-slate-50 flex flex-col">
            <header class="border-b border-slate-800 bg-slate-900/80 backdrop-blur px-6 py-4 flex items-center justify-between">
                <div>
                    <h1 class="text-lg font-semibold tracking-tight">"FANUC Robot Control"</h1>
                    <p class="text-xs text-slate-400">"Real-time robot control using eventwork_sync"</p>
                </div>
                <div class="flex items-center gap-2">
                    <input
                        type="text"
                        class="px-2 py-1 rounded bg-slate-800 border border-slate-700 text-xs w-40 focus:outline-none focus:ring-1 focus:ring-emerald-500"
                        prop:value=host
                        on:input=move |ev| set_host.set(event_target_value(&ev))
                        placeholder="Host"
                    />
                    <input
                        type="text"
                        class="px-2 py-1 rounded bg-slate-800 border border-slate-700 text-xs w-20 focus:outline-none focus:ring-1 focus:ring-emerald-500"
                        prop:value=port
                        on:input=move |ev| set_port.set(event_target_value(&ev))
                        placeholder="Port"
                    />
                    <button
                        class="px-3 py-1.5 rounded bg-emerald-500 text-xs font-medium text-slate-950 hover:bg-emerald-400 transition"
                        on:click=on_connect
                    >
                        "Connect"
                    </button>
                </div>
            </header>

            <main class="flex-1 overflow-hidden">
                <Show
                    when=move || ws_url.get().is_some()
                    fallback=move || view! {
                        <div class="h-full w-full flex items-center justify-center text-slate-400 text-sm">
                            <p>"Enter a host/port and click Connect to start controlling the robot."</p>
                        </div>
                    }
                >
                    {
                        let registry = registry.clone();
                        move || {
                            ws_url.get().map(|url| {
                                // Create WebSocket connection for the FANUC client
                                let UseWebSocketReturn {
                                    ready_state,
                                    message: raw_message,
                                    send: raw_send,
                                    ..
                                } = use_websocket_with_options::<
                                    NetworkPacket,
                                    NetworkPacket,
                                    EventworkBincodeCodec,
                                    (),
                                    DummyEncoder,
                                >(
                                    url,
                                    UseWebSocketOptions::default().immediate(true),
                                );

                                // Deserialize NetworkPacket to SyncServerMessage
                                let message = Signal::derive(move || {
                                    raw_message.with(|packet_opt| {
                                        packet_opt.as_ref().and_then(|packet| {
                                            bincode::serde::decode_from_slice::<SyncServerMessage, _>(
                                                &packet.data,
                                                bincode::config::standard()
                                            ).ok().map(|(msg, _)| msg)
                                        })
                                    })
                                });

                                // Create send function that wraps messages in NetworkPacket
                                let send = move |msg: &SyncClientMessage| {
                                    let packet = NetworkPacket {
                                        type_name: std::any::type_name::<SyncClientMessage>().to_string(),
                                        schema_hash: 0,
                                        data: bincode::serde::encode_to_vec(msg, bincode::config::standard()).unwrap(),
                                    };
                                    raw_send(&packet);
                                };

                                // Create SyncClient for the FANUC client
                                let client = Arc::new(SyncClient::new(
                                    move |msg: SyncClientMessage| {
                                        send(&msg);
                                    },
                                    registry.clone(),
                                ));

                                // Provide SyncClient via context for JogControls
                                provide_context(client.clone());

                                // Create reactive signals for robot data
                                let robot_position = RwSignal::new(None::<RobotPosition>);
                                let robot_status = RwSignal::new(None::<RobotStatus>);

                                // Provide robot data signals via context
                                provide_context(robot_position);
                                provide_context(robot_status);

                                // Subscribe to RobotPosition and RobotStatus when connected
                                {
                                    let client = client.clone();
                                    Effect::new(move |_| {
                                        if ready_state.get() == ConnectionReadyState::Open {
                                            // Subscribe to RobotPosition
                                            client.send_raw(SyncClientMessage::Subscription(SubscriptionRequest {
                                                subscription_id: 1,
                                                component_type: "RobotPosition".to_string(),
                                                entity: None,
                                            }));
                                            // Subscribe to RobotStatus
                                            client.send_raw(SyncClientMessage::Subscription(SubscriptionRequest {
                                                subscription_id: 2,
                                                component_type: "RobotStatus".to_string(),
                                                entity: None,
                                            }));
                                        }
                                    });
                                }

                                // Handle incoming server messages and update reactive signals
                                {
                                    let client = client.clone();
                                    Effect::new(move |_| {
                                        message.with(|msg_opt| {
                                            if let Some(msg) = msg_opt {
                                                // Let SyncClient handle the message for mutation tracking
                                                client.handle_server_message(msg);

                                                // Update our reactive signals for UI display
                                                if let SyncServerMessage::SyncBatch(batch) = msg {
                                                    for item in &batch.items {
                                                        match item {
                                                            SyncItem::Snapshot { component_type, value, .. }
                                                            | SyncItem::Update { component_type, value, .. } => {
                                                                match component_type.as_str() {
                                                                    "RobotPosition" => {
                                                                        if let Ok((pos, _)) = bincode::serde::decode_from_slice::<RobotPosition, _>(
                                                                            value,
                                                                            bincode::config::standard()
                                                                        ) {
                                                                            robot_position.set(Some(pos));
                                                                        }
                                                                    }
                                                                    "RobotStatus" => {
                                                                        if let Ok((status, _)) = bincode::serde::decode_from_slice::<RobotStatus, _>(
                                                                            value,
                                                                            bincode::config::standard()
                                                                        ) {
                                                                            robot_status.set(Some(status));
                                                                        }
                                                                    }
                                                                    _ => {}
                                                                }
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    });
                                }

                                view! {
                                    <div class="flex-1 flex">
                                        <div class="flex-1 p-6 overflow-auto">
                                            <div class="max-w-4xl mx-auto space-y-6">
                                                <RobotStatusDisplay />
                                                <PositionDisplay />
                                                <JogControls />
                                            </div>
                                        </div>
                                        <div class="w-1/2 border-l border-slate-800">
                                            <DevTools ws_url=url registry=registry.clone() />
                                        </div>
                                    </div>
                                }
                            })
                        }
                    }
                </Show>
            </main>
        </div>
    }
}

#[component]
fn RobotStatusDisplay() -> impl IntoView {
    let robot_status = use_context::<RwSignal<Option<RobotStatus>>>()
        .expect("RobotStatus signal should be provided");

    view! {
        <div class="bg-slate-900 rounded-lg border border-slate-800 p-4">
            <h2 class="text-sm font-semibold mb-3">"Robot Status"</h2>
            <div class="grid grid-cols-2 gap-3 text-xs">
                <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full" class:bg-emerald-500=move || robot_status.get().map(|s| s.servo_ready).unwrap_or(false) class:bg-slate-600=move || !robot_status.get().map(|s| s.servo_ready).unwrap_or(false)></div>
                    <span class="text-slate-400">"Servo Ready"</span>
                </div>
                <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full" class:bg-emerald-500=move || robot_status.get().map(|s| s.tp_enabled).unwrap_or(false) class:bg-slate-600=move || !robot_status.get().map(|s| s.tp_enabled).unwrap_or(false)></div>
                    <span class="text-slate-400">"TP Enabled"</span>
                </div>
                <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full" class:bg-emerald-500=move || robot_status.get().map(|s| s.in_motion).unwrap_or(false) class:bg-slate-600=move || !robot_status.get().map(|s| s.in_motion).unwrap_or(false)></div>
                    <span class="text-slate-400">"In Motion"</span>
                </div>
                <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full" class:bg-emerald-500=move || robot_status.get().map(|s| s.error_code.is_none()).unwrap_or(true) class:bg-red-500=move || robot_status.get().map(|s| s.error_code.is_some()).unwrap_or(false)></div>
                    <span class="text-slate-400">{move || if robot_status.get().map(|s| s.error_code.is_some()).unwrap_or(false) { "Error" } else { "No Errors" }}</span>
                </div>
            </div>
        </div>
    }
}

#[component]
fn PositionDisplay() -> impl IntoView {
    let robot_position = use_context::<RwSignal<Option<RobotPosition>>>()
        .expect("RobotPosition signal should be provided");

    view! {
        <div class="bg-slate-900 rounded-lg border border-slate-800 p-4">
            <h2 class="text-sm font-semibold mb-3">"Robot Position"</h2>
            <div class="grid grid-cols-3 gap-4 text-xs">
                <div>
                    <div class="text-slate-400 mb-1">"X"</div>
                    <div class="font-mono text-emerald-400">{move || format!("{:.2} mm", robot_position.get().map(|p| p.x).unwrap_or(0.0))}</div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"Y"</div>
                    <div class="font-mono text-emerald-400">{move || format!("{:.2} mm", robot_position.get().map(|p| p.y).unwrap_or(0.0))}</div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"Z"</div>
                    <div class="font-mono text-emerald-400">{move || format!("{:.2} mm", robot_position.get().map(|p| p.z).unwrap_or(0.0))}</div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"W"</div>
                    <div class="font-mono text-blue-400">{move || format!("{:.2}°", robot_position.get().map(|p| p.w).unwrap_or(0.0))}</div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"P"</div>
                    <div class="font-mono text-blue-400">{move || format!("{:.2}°", robot_position.get().map(|p| p.p).unwrap_or(0.0))}</div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"R"</div>
                    <div class="font-mono text-blue-400">{move || format!("{:.2}°", robot_position.get().map(|p| p.r).unwrap_or(0.0))}</div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn JogControls() -> impl IntoView {
    view! {
        <div class="bg-slate-900 rounded-lg border border-slate-800 p-4">
            <h2 class="text-sm font-semibold mb-3">"Jog Controls"</h2>
            <div class="space-y-4">
                <div>
                    <div class="text-xs text-slate-400 mb-2">"Cartesian (mm)"</div>
                    <div class="grid grid-cols-3 gap-2">
                        <JogButton axis="X" />
                        <JogButton axis="Y" />
                        <JogButton axis="Z" />
                    </div>
                </div>
                <div>
                    <div class="text-xs text-slate-400 mb-2">"Orientation (deg)"</div>
                    <div class="grid grid-cols-3 gap-2">
                        <JogButton axis="W" />
                        <JogButton axis="P" />
                        <JogButton axis="R" />
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn JogButton(axis: &'static str) -> impl IntoView {
    // Get SyncClient from context
    let sync_client = expect_context::<Arc<SyncClient>>();

    // Parse axis string to RobotAxis enum
    let robot_axis = match axis {
        "X" => RobotAxis::X,
        "Y" => RobotAxis::Y,
        "Z" => RobotAxis::Z,
        "W" => RobotAxis::W,
        "P" => RobotAxis::P,
        "R" => RobotAxis::R,
        _ => RobotAxis::X,
    };

    // Clone Arc for each closure
    let sync_client_pos = sync_client.clone();
    let on_jog_positive = move |_| {
        use fanuc_shared::axis::JogDirection;
        log::info!("Jog {} positive", axis);

        // Create JogCommand struct
        let jog_cmd = JogCommand {
            axis: robot_axis,
            direction: JogDirection::Positive,
            speed: 10.0,
            distance: 1.0,
        };

        // Convert to JSON and send mutation
        if let Ok(value) = to_value(&jog_cmd) {
            sync_client_pos.mutate(
                SerializableEntity::DANGLING,
                "JogCommand",
                value,
            );
        }
    };

    let sync_client_neg = sync_client.clone();
    let on_jog_negative = move |_| {
        use fanuc_shared::axis::JogDirection;
        log::info!("Jog {} negative", axis);

        // Create JogCommand struct
        let jog_cmd = JogCommand {
            axis: robot_axis,
            direction: JogDirection::Negative,
            speed: 10.0,
            distance: 1.0,
        };

        // Convert to JSON and send mutation
        if let Ok(value) = to_value(&jog_cmd) {
            sync_client_neg.mutate(
                SerializableEntity::DANGLING,
                "JogCommand",
                value,
            );
        }
    };

    view! {
        <div class="flex flex-col gap-1">
            <button
                class="px-2 py-1.5 rounded bg-emerald-600 text-xs font-medium hover:bg-emerald-500 transition"
                on:click=on_jog_positive
            >
                {format!("{}+", axis)}
            </button>
            <button
                class="px-2 py-1.5 rounded bg-red-600 text-xs font-medium hover:bg-red-500 transition"
                on:click=on_jog_negative
            >
                {format!("{}-", axis)}
            </button>
        </div>
    }
}

