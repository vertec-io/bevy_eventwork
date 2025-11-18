use eventwork_devtools::DevTools;
use eventwork_sync::client_registry::ComponentTypeRegistry;
use leptos::prelude::*;
use reactive_graph::traits::Get;

// Import shared component types
use demo_shared::{DemoCounter, DemoFlag, ParentEntity, ChildEntities};

fn main() {
    console_error_panic_hook::set_once();
    _ = console_log::init_with_level(log::Level::Debug);

    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (host, set_host) = signal("127.0.0.1".to_string());
    let (port, set_port) = signal("8081".to_string());
    let (ws_url, set_ws_url) = signal(None::<&'static str>);

    // Create type registry and register component types
    let mut registry = ComponentTypeRegistry::new();
    registry.register::<DemoCounter>();
    registry.register::<DemoFlag>();
    registry.register::<ParentEntity>();
    registry.register::<ChildEntities>();

    let on_connect = move |_| {
        let url_owned = format!("ws://{}:{}", host.get(), port.get());
        let ws_url_static: &'static str = Box::leak(url_owned.into_boxed_str());
        set_ws_url.set(Some(ws_url_static));
    };

    view! {
        <div class="min-h-screen w-screen bg-slate-950 text-slate-50 flex flex-col">
            <header class="border-b border-slate-800 bg-slate-900/80 backdrop-blur px-6 py-4 flex items-center justify-between">
                <div>
                    <h1 class="text-lg font-semibold tracking-tight">"Eventwork DevTools Demo"</h1>
                    <p class="text-xs text-slate-400">"Configure the WebSocket endpoint, then launch the DevTools inspector."</p>
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
                        <div class="h-full flex items-center justify-center text-slate-400 text-sm">
                            <p>"Enter a host/port and click Connect to start the DevTools session."</p>
                        </div>
                    }
                >
                    {
                        let registry = registry.clone();
                        move || {
                            ws_url.get().map(|url| {
                                view! { <DevTools ws_url=url registry=registry.clone() /> }
                            })
                        }
                    }
                </Show>
            </main>
        </div>
    }
}

