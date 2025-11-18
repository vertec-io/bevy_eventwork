use eventwork_devtools::{DevTools, DevToolsMode};
use eventwork_sync::{
    client_registry::ComponentTypeRegistry,
    client_sync::SyncClient,
    SyncClientMessage,
    SyncServerMessage,
    SyncItem,
    SubscriptionRequest,
    SerializableEntity,
};
use eventwork_common::{codec::EventworkBincodeCodec, NetworkPacket};
use leptos::prelude::*;
use leptos_use::{use_websocket_with_options, DummyEncoder, UseWebSocketOptions, UseWebSocketReturn, core::ConnectionReadyState};
use reactive_graph::traits::Get;
use std::sync::Arc;

// Import shared component types
use fanuc_real_shared::{RobotPosition, RobotStatus, JointAngles, RobotInfo, JogCommand, JogAxis, JogDirection, MotionCommand};

fn main() {
    console_error_panic_hook::set_once();
    _ = console_log::init_with_level(log::Level::Debug);

    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> AnyView {
    // Check if we're in devtools-only mode
    let is_devtools_only = leptos::web_sys::window()
        .and_then(|w| w.location().search().ok())
        .map(|search| search.contains("devtools=1"))
        .unwrap_or(false);

    if is_devtools_only {
        // Render only DevTools in embedded mode
        let mut registry = ComponentTypeRegistry::new();
        registry.register::<RobotPosition>();
        registry.register::<RobotStatus>();
        registry.register::<JointAngles>();
        registry.register::<RobotInfo>();
        registry.register::<JogCommand>();
        registry.register::<MotionCommand>();

        // Use default WebSocket URL for devtools-only mode
        let ws_url = "ws://127.0.0.1:8082";

        return view! {
            <div class="min-h-screen w-screen bg-slate-950">
                <DevTools ws_url=ws_url registry=registry mode=DevToolsMode::Embedded />
            </div>
        }.into_any();
    }

    // Normal app mode
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
    // MotionCommand contains dto::Instruction which is WASM-compatible (DTO feature has no tokio/mio)
    registry.register::<MotionCommand>();

    let on_connect = move |_| {
        let url_owned = format!("ws://{}:{}", host.get(), port.get());
        let ws_url_static: &'static str = Box::leak(url_owned.into_boxed_str());
        set_ws_url.set(Some(ws_url_static));
    };

    view! {
        <div class="min-h-screen w-screen bg-slate-950 text-slate-50 flex flex-col">
            <header class="border-b border-slate-800 bg-slate-900/80 backdrop-blur px-6 py-4 flex items-center justify-between">
                <div>
                    <h1 class="text-lg font-semibold tracking-tight">"FANUC Real Robot Control"</h1>
                    <p class="text-xs text-slate-400">"Real FANUC simulator control using eventwork_sync"</p>
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

                                // Provide SyncClient via context
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
                                    </div>
                                    // DevTools as floating widget (default mode)
                                    <DevTools ws_url=url registry=registry.clone() />
                                }
                            })
                        }
                    }
                </Show>
            </main>
        </div>
    }.into_any()
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
                    <div class="w-2 h-2 rounded-full" class:bg-amber-500=move || robot_status.get().map(|s| s.in_motion).unwrap_or(false) class:bg-slate-600=move || !robot_status.get().map(|s| s.in_motion).unwrap_or(false)></div>
                    <span class="text-slate-400">"In Motion"</span>
                </div>
                <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full" class:bg-red-500=move || robot_status.get().and_then(|s| s.error_message.clone()).is_some() class:bg-slate-600=move || robot_status.get().and_then(|s| s.error_message.clone()).is_none()></div>
                    <span class="text-slate-400">"Error"</span>
                </div>
            </div>
            <Show when=move || robot_status.get().and_then(|s| s.error_message.clone()).is_some()>
                <div class="mt-3 p-2 bg-red-500/10 border border-red-500/20 rounded text-xs text-red-400">
                    {move || robot_status.get().and_then(|s| s.error_message.clone()).unwrap_or_default()}
                </div>
            </Show>
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
            <div class="grid grid-cols-3 gap-3 text-xs">
                <div>
                    <div class="text-slate-400 mb-1">"X"</div>
                    <div class="font-mono text-emerald-400">{move || format!("{:.2}", robot_position.get().map(|p| p.x).unwrap_or(0.0))}</div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"Y"</div>
                    <div class="font-mono text-emerald-400">{move || format!("{:.2}", robot_position.get().map(|p| p.y).unwrap_or(0.0))}</div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"Z"</div>
                    <div class="font-mono text-emerald-400">{move || format!("{:.2}", robot_position.get().map(|p| p.z).unwrap_or(0.0))}</div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"W"</div>
                    <div class="font-mono text-emerald-400">{move || format!("{:.2}", robot_position.get().map(|p| p.w).unwrap_or(0.0))}</div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"P"</div>
                    <div class="font-mono text-emerald-400">{move || format!("{:.2}", robot_position.get().map(|p| p.p).unwrap_or(0.0))}</div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"R"</div>
                    <div class="font-mono text-emerald-400">{move || format!("{:.2}", robot_position.get().map(|p| p.r).unwrap_or(0.0))}</div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn JogControls() -> impl IntoView {
    let client = use_context::<Arc<SyncClient>>()
        .expect("SyncClient should be provided");

    view! {
        <div class="bg-slate-900 rounded-lg border border-slate-800 p-4">
            <h2 class="text-sm font-semibold mb-3">"Jog Controls"</h2>
            <div class="space-y-4">
                <div>
                    <div class="text-xs text-slate-400 mb-2">"Cartesian (mm)"</div>
                    <div class="grid grid-cols-3 gap-2">
                        <JogButton axis=JogAxis::X client=client.clone() />
                        <JogButton axis=JogAxis::Y client=client.clone() />
                        <JogButton axis=JogAxis::Z client=client.clone() />
                    </div>
                </div>
                <div>
                    <div class="text-xs text-slate-400 mb-2">"Orientation (deg)"</div>
                    <div class="grid grid-cols-3 gap-2">
                        <JogButton axis=JogAxis::W client=client.clone() />
                        <JogButton axis=JogAxis::P client=client.clone() />
                        <JogButton axis=JogAxis::R client=client.clone() />
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn JogButton(axis: JogAxis, client: Arc<SyncClient>) -> impl IntoView {
    let axis_str = match axis {
        JogAxis::X => "X",
        JogAxis::Y => "Y",
        JogAxis::Z => "Z",
        JogAxis::W => "W",
        JogAxis::P => "P",
        JogAxis::R => "R",
    };

    // Clone for each closure
    let client_pos = client.clone();
    let client_neg = client.clone();

    let on_jog_positive = move |_| {
        log::info!("Jog {} positive", axis_str);

        let jog_cmd = JogCommand {
            axis,
            direction: JogDirection::Positive,
            distance: 10.0,  // 10mm or 10 degrees
            speed: 50.0,     // 50mm/s or 50deg/s
        };

        // Convert to JSON and send mutation
        if let Ok(value) = serde_json::to_value(&jog_cmd) {
            client_pos.mutate(
                SerializableEntity::DANGLING,
                "JogCommand",
                value,
            );
        }
    };

    let on_jog_negative = move |_| {
        log::info!("Jog {} negative", axis_str);

        let jog_cmd = JogCommand {
            axis,
            direction: JogDirection::Negative,
            distance: 10.0,  // 10mm or 10 degrees
            speed: 50.0,     // 50mm/s or 50deg/s
        };

        // Convert to JSON and send mutation
        if let Ok(value) = serde_json::to_value(&jog_cmd) {
            client_neg.mutate(
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
                {format!("{}+", axis_str)}
            </button>
            <button
                class="px-2 py-1.5 rounded bg-red-600 text-xs font-medium hover:bg-red-500 transition"
                on:click=on_jog_negative
            >
                {format!("{}-", axis_str)}
            </button>
        </div>
    }
}
