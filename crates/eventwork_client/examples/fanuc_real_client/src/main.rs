use eventwork_client::{
    ClientRegistryBuilder, DevTools, SyncProvider, impl_sync_component, use_sync_component,
};
use fanuc_real_shared::{RobotPosition, RobotStatus, JointAngles, RobotInfo};
use leptos::prelude::*;

// Implement SyncComponent for all types
impl_sync_component!(RobotPosition);
impl_sync_component!(RobotStatus);
impl_sync_component!(JointAngles);
impl_sync_component!(RobotInfo);

fn main() {
    console_error_panic_hook::set_once();
    _ = console_log::init_with_level(log::Level::Debug);

    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (host, set_host) = signal("127.0.0.1".to_string());
    let (port, set_port) = signal("8082".to_string());
    let (ws_url, set_ws_url) = signal(None::<String>);

    // Create type registry and register component types
    let registry = ClientRegistryBuilder::new()
        .register::<RobotPosition>()
        .register::<RobotStatus>()
        .register::<JointAngles>()
        .register::<RobotInfo>()
        .build();

    let on_connect = move |_| {
        let url = format!("ws://{}:{}", host.get(), port.get());
        set_ws_url.set(Some(url));
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
                    {move || {
                        ws_url.get().map(|url| {
                            view! {
                                <SyncProvider url=url registry=registry.clone()>
                                    <div class="flex-1 flex">
                                        <div class="flex-1 p-6 overflow-auto">
                                            <div class="max-w-4xl mx-auto space-y-6">
                                                <RobotStatusDisplay />
                                                <PositionDisplay />
                                                <JointAnglesDisplay />
                                            </div>
                                        </div>
                                        <div class="w-1/2 border-l border-slate-800">
                                            <DevTools />
                                        </div>
                                    </div>
                                </SyncProvider>
                            }
                        })
                    }}
                </Show>
            </main>
        </div>
    }
}

#[component]
fn RobotStatusDisplay() -> impl IntoView {
    let robot_statuses = use_sync_component::<RobotStatus>();

    // Get the first robot status (assuming single robot)
    let robot_status = move || {
        robot_statuses.get()
            .values()
            .next()
            .cloned()
    };

    view! {
        <div class="bg-slate-900 rounded-lg border border-slate-800 p-4">
            <h2 class="text-sm font-semibold mb-3">"Robot Status"</h2>
            <div class="grid grid-cols-2 gap-3 text-xs">
                <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full"
                        class:bg-emerald-500=move || robot_status().map(|s| s.servo_ready).unwrap_or(false)
                        class:bg-slate-600=move || !robot_status().map(|s| s.servo_ready).unwrap_or(false)
                    ></div>
                    <span class="text-slate-400">"Servo Ready"</span>
                </div>
                <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full"
                        class:bg-emerald-500=move || robot_status().map(|s| s.tp_enabled).unwrap_or(false)
                        class:bg-slate-600=move || !robot_status().map(|s| s.tp_enabled).unwrap_or(false)
                    ></div>
                    <span class="text-slate-400">"TP Enabled"</span>
                </div>
                <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full"
                        class:bg-amber-500=move || robot_status().map(|s| s.in_motion).unwrap_or(false)
                        class:bg-slate-600=move || !robot_status().map(|s| s.in_motion).unwrap_or(false)
                    ></div>
                    <span class="text-slate-400">"In Motion"</span>
                </div>
            </div>
        </div>
    }
}

#[component]
fn PositionDisplay() -> impl IntoView {
    let robot_positions = use_sync_component::<RobotPosition>();

    // Get the first robot position (assuming single robot)
    let robot_position = move || {
        robot_positions.get()
            .values()
            .next()
            .cloned()
    };

    view! {
        <div class="bg-slate-900 rounded-lg border border-slate-800 p-4">
            <h2 class="text-sm font-semibold mb-3">"Robot Position (Cartesian)"</h2>
            <div class="grid grid-cols-3 gap-3 text-xs">
                <div>
                    <div class="text-slate-400 mb-1">"X (mm)"</div>
                    <div class="font-mono text-emerald-400">
                        {move || format!("{:.2}", robot_position().map(|p| p.x).unwrap_or(0.0))}
                    </div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"Y (mm)"</div>
                    <div class="font-mono text-emerald-400">
                        {move || format!("{:.2}", robot_position().map(|p| p.y).unwrap_or(0.0))}
                    </div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"Z (mm)"</div>
                    <div class="font-mono text-emerald-400">
                        {move || format!("{:.2}", robot_position().map(|p| p.z).unwrap_or(0.0))}
                    </div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"W (deg)"</div>
                    <div class="font-mono text-emerald-400">
                        {move || format!("{:.2}", robot_position().map(|p| p.w).unwrap_or(0.0))}
                    </div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"P (deg)"</div>
                    <div class="font-mono text-emerald-400">
                        {move || format!("{:.2}", robot_position().map(|p| p.p).unwrap_or(0.0))}
                    </div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"R (deg)"</div>
                    <div class="font-mono text-emerald-400">
                        {move || format!("{:.2}", robot_position().map(|p| p.r).unwrap_or(0.0))}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn JointAnglesDisplay() -> impl IntoView {
    let joint_angles = use_sync_component::<JointAngles>();

    // Get the first joint angles (assuming single robot)
    let joints = move || {
        joint_angles.get()
            .values()
            .next()
            .cloned()
    };

    view! {
        <div class="bg-slate-900 rounded-lg border border-slate-800 p-4">
            <h2 class="text-sm font-semibold mb-3">"Joint Angles"</h2>
            <div class="grid grid-cols-3 gap-3 text-xs">
                <div>
                    <div class="text-slate-400 mb-1">"J1 (deg)"</div>
                    <div class="font-mono text-blue-400">
                        {move || format!("{:.2}", joints().map(|j| j.j1).unwrap_or(0.0))}
                    </div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"J2 (deg)"</div>
                    <div class="font-mono text-blue-400">
                        {move || format!("{:.2}", joints().map(|j| j.j2).unwrap_or(0.0))}
                    </div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"J3 (deg)"</div>
                    <div class="font-mono text-blue-400">
                        {move || format!("{:.2}", joints().map(|j| j.j3).unwrap_or(0.0))}
                    </div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"J4 (deg)"</div>
                    <div class="font-mono text-blue-400">
                        {move || format!("{:.2}", joints().map(|j| j.j4).unwrap_or(0.0))}
                    </div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"J5 (deg)"</div>
                    <div class="font-mono text-blue-400">
                        {move || format!("{:.2}", joints().map(|j| j.j5).unwrap_or(0.0))}
                    </div>
                </div>
                <div>
                    <div class="text-slate-400 mb-1">"J6 (deg)"</div>
                    <div class="font-mono text-blue-400">
                        {move || format!("{:.2}", joints().map(|j| j.j6).unwrap_or(0.0))}
                    </div>
                </div>
            </div>
        </div>
    }
}

