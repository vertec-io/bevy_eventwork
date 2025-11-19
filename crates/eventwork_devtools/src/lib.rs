use leptos::prelude::*;
use reactive_graph::traits::{Get, Update};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

use eventwork_sync::{
    MutationResponse,
    SerializableEntity,
    SyncClientMessage,
    SyncServerMessage,
    client_registry::ComponentTypeRegistry,
    client_sync::SyncClient,
};

/// Leptos-reactive wrapper around `SyncClient` for use in Leptos applications.
///
/// This provides the same API as `SyncClient` but with reactive signals for
/// tracking mutation state in Leptos components.
#[derive(Clone)]
pub struct DevtoolsSync {
    client: Arc<SyncClient>,
    mutations: RwSignal<HashMap<u64, eventwork_sync::client_sync::MutationState>>,
}

// Re-export MutationState from eventwork_sync for convenience
pub use eventwork_sync::client_sync::MutationState;

// Re-export DevTools UI components for WASM targets
#[cfg(target_arch = "wasm32")]
pub use ui::{DevTools, DevToolsMode};

/// General-purpose sync hook for wiring the eventwork_sync wire protocol
/// into an arbitrary transport (typically a WebSocket using eventwork's
/// binary codec).
///
/// The `send` closure is responsible for serializing and transmitting
/// `SyncClientMessage` values. This keeps the devtools crate agnostic of
/// any particular WebSocket or HTTP client implementation.
///
/// The `registry` is used to serialize mutations from JSON back to the
/// concrete component types expected by the server.
pub fn use_sync(
    send: impl Fn(SyncClientMessage) + Send + Sync + 'static,
    registry: ComponentTypeRegistry,
) -> DevtoolsSync {
    let client = Arc::new(SyncClient::new(send, registry));
    let mutations = RwSignal::new(HashMap::new());

    DevtoolsSync {
        client,
        mutations,
    }
}

impl DevtoolsSync {
    /// Send a raw `SyncClientMessage` without any local bookkeeping.
    ///
    /// This is useful for subscription management or other operations
    /// that don't need per-request client-side tracking.
    pub fn send_raw(&self, message: SyncClientMessage) {
        self.client.send_raw(message);
    }

    /// Read-only view of all tracked mutations keyed by `request_id`.
    pub fn mutations(&self) -> RwSignal<HashMap<u64, MutationState>> {
        self.mutations
    }

    /// Convenience accessor for a single mutation state, if known.
    pub fn mutation_state(&self, request_id: u64) -> Option<MutationState> {
        self.mutations.get().get(&request_id).cloned()
    }

    /// Queue a new mutation for `(entity, component_type)` with the
    /// provided JSON value. Returns the generated `request_id` that will
    /// be echoed back by the server in its `MutationResponse`.
    pub fn mutate(
        &self,
        entity: SerializableEntity,
        component_type: impl Into<String>,
        value: JsonValue,
    ) -> u64 {
        // Delegate to SyncClient
        let request_id = self.client.mutate(entity, component_type, value);

        // Track in reactive signal for Leptos
        self.mutations.update(|map| {
            map.insert(request_id, MutationState::new_pending(request_id));
        });

        request_id
    }

    /// Handle a server-side message, updating mutation state when a
    /// `MutationResponse` is observed.
    pub fn handle_server_message(&self, message: &SyncServerMessage) {
        // Delegate to SyncClient
        self.client.handle_server_message(message);

        // Sync the mutation state to our reactive signal
        self.sync_mutations_from_client();
    }

    /// Helper to handle a `MutationResponse` directly, for cases where
    /// the transport layer already demultiplexes server messages.
    pub fn handle_mutation_response(&self, response: &MutationResponse) {
        // Delegate to SyncClient
        self.client.handle_mutation_response(response);

        // Sync the mutation state to our reactive signal
        self.sync_mutations_from_client();
    }

    /// Sync mutation state from the underlying SyncClient to the reactive signal.
    fn sync_mutations_from_client(&self) {
        let client_mutations = self.client.mutations();
        self.mutations.set(client_mutations);
    }

    /// Get a reference to the underlying SyncClient.
    pub fn client(&self) -> &SyncClient {
        &self.client
    }
}


// Re-export core wire-level types so downstream tools can depend on this
// crate alone for typical sync workflows.
pub use eventwork_sync::{
    MutateComponent as SyncMutateComponent,
    MutationResponse as SyncMutationResponse,
    MutationStatus as SyncMutationStatus,
    SerializableEntity as SyncSerializableEntity,
    SyncBatch,
    SyncClientMessage as SyncClientMsg,
    SyncItem,
    SyncServerMessage as SyncServerMsg,
    SubscriptionRequest,
    UnsubscribeRequest,
};

#[cfg(target_arch = "wasm32")]
pub mod ui {
    use super::*;
    use eventwork_common::codec::EventworkBincodeCodec;
    use eventwork_common::NetworkPacket;
    use leptos_use::{
        core::ConnectionReadyState,
        use_websocket_with_options,
        DummyEncoder,
        UseWebSocketOptions,
        UseWebSocketReturn,
    };
    use leptos::web_sys::console;

    fn entity_label(id: u64, components: &HashMap<String, JsonValue>) -> String {
        for value in components.values() {
            if let JsonValue::Object(obj) = value {
                if let Some(JsonValue::String(name)) =
                    obj.get("name").or_else(|| obj.get("label"))
                {
                    return format!("{name} · #{id}");
                }
            }
        }
        format!("Entity #{id}")
    }

    fn parse_number_like(original: &serde_json::Number, text: &str) -> Option<serde_json::Number> {
        if original.is_i64() {
            text.parse::<i64>().ok().map(serde_json::Number::from)
        } else if original.is_u64() {
            text.parse::<u64>().ok().map(serde_json::Number::from)
        } else {
            text
                .parse::<f64>()
                .ok()
                .and_then(serde_json::Number::from_f64)
        }
    }

    fn apply_field_update(
        entities: RwSignal<HashMap<u64, HashMap<String, JsonValue>>>,
        sync: RwSignal<DevtoolsSync>,
        entity_bits: u64,
        component_type: String,
        field_name: String,
        new_value: JsonValue,
    ) {
        let mut updated_component: Option<JsonValue> = None;

        entities.update(|map| {
            if let Some(components) = map.get_mut(&entity_bits) {
                if let Some(component_value) = components.get_mut(&component_type) {
                    match component_value {
                        JsonValue::Object(obj) => {
                            obj.insert(field_name.clone(), new_value.clone());
                        }
                        _ => {
                            *component_value = new_value.clone();
                        }
                    }
                    updated_component = Some(component_value.clone());
                }
            }
        });

        if let Some(component_json) = updated_component {
            sync.get().mutate(
                SerializableEntity { bits: entity_bits },
                component_type,
                component_json,
            );
        }
    }

    fn component_editor(
        entity_bits: u64,
        component_type: String,
        entities: RwSignal<HashMap<u64, HashMap<String, JsonValue>>>,
        sync: RwSignal<DevtoolsSync>,
    ) -> impl IntoView {
        let component_type_for_fields = component_type.clone();
        let fields = move || {
            entities
                .get()
                .get(&entity_bits)
                .and_then(|components| components.get(&component_type_for_fields))
                .and_then(|value| {
                    if let JsonValue::Object(obj) = value {
                        let mut v: Vec<(String, JsonValue)> =
                            obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                        v.sort_by_key(|(k, _)| k.clone());
                        Some(v)
                    } else {
                        None
                    }
                })
                .unwrap_or_default()
        };

        view! {
            <div class="space-y-2">
                <For
                    each=fields
                    key=|(field, _)| field.clone()
                    children=move |(field_name, field_value): (String, JsonValue)| {
                        let entities_for_handler = entities;
                        let sync_for_handler = sync;
                        let component_type_for_handler = component_type.clone();
                        let field_for_handler = field_name.clone();
                        let entity_bits_for_handler = entity_bits;

                        let field_view: AnyView = match field_value {
                            JsonValue::Bool(b) => view! {
                                <div class="flex items-center justify-between gap-2">
                                    <span class="text-[11px] text-slate-300">{field_name.clone()}</span>
                                    <input
                                        type="checkbox"
                                        class="h-3 w-3 rounded border-slate-600 bg-slate-950"
                                        prop:checked=b
                                        on:input=move |ev| {
                                            let value = event_target_checked(&ev);
                                            apply_field_update(
                                                entities_for_handler,
                                                sync_for_handler,
                                                entity_bits_for_handler,
                                                component_type_for_handler.clone(),
                                                field_for_handler.clone(),
                                                JsonValue::Bool(value),
                                            );
                                        }
                                    />
                                </div>
                            }.into_any(),
                            JsonValue::Number(num) => {
                                let initial = num.to_string();
                                view! {
                                    <div class="space-y-1">
                                        <div class="text-[11px] text-slate-300">{field_name.clone()}</div>
                                        <input
                                            class="w-full rounded-md bg-slate-950/70 border border-slate-700 px-2 py-1 text-[11px] focus:outline-none focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500"
                                            value=initial
                                            on:change=move |ev| {
                                                let raw = event_target_value(&ev);
                                                if let Some(num) = parse_number_like(&num, &raw) {
                                                    apply_field_update(
                                                        entities_for_handler,
                                                        sync_for_handler,
                                                        entity_bits_for_handler,
                                                        component_type_for_handler.clone(),
                                                        field_for_handler.clone(),
                                                        JsonValue::Number(num),
                                                    );
                                                }
                                            }
                                        />
                                    </div>
                                }.into_any()
                            }
                            JsonValue::String(s) => view! {
                                <div class="space-y-1">
                                    <div class="text-[11px] text-slate-300">{field_name.clone()}</div>
                                    <input
                                        class="w-full rounded-md bg-slate-950/70 border border-slate-700 px-2 py-1 text-[11px] focus:outline-none focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500"
                                        value=s
                                        on:change=move |ev| {
                                            let raw = event_target_value(&ev);
                                            apply_field_update(
                                                entities_for_handler,
                                                sync_for_handler,
                                                entity_bits_for_handler,
                                                component_type_for_handler.clone(),
                                                field_for_handler.clone(),
                                                JsonValue::String(raw),
                                            );
                                        }
                                    />
                                </div>
                            }.into_any(),
                            other => {
                                let json = serde_json::to_string_pretty(&other).unwrap_or_default();
                                view! {
                                    <div class="space-y-1">
                                        <div class="text-[11px] text-slate-300">{field_name.clone()}</div>
                                        <pre class="mt-0.5 bg-slate-950/60 border border-slate-800 rounded p-1 font-mono text-[10px] whitespace-pre-wrap break-all">{json}</pre>
                                    </div>
                                }.into_any()
                            }
                        };

                        field_view
                    }
                />
            </div>
        }
    }

    /// Display mode for the DevTools component
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum DevToolsMode {
        /// Floating widget in lower-left corner (default)
        Widget,
        /// Full-page embedded view
        Embedded,
    }

    impl Default for DevToolsMode {
        fn default() -> Self {
            Self::Widget
        }
    }

    /// High-level DevTools surface: given a WebSocket URL speaking the
    /// `eventwork_sync` wire protocol, render a modern Tailwind-powered
    /// inspector + mutation console.
    ///
    /// # Parameters
    /// - `ws_url`: WebSocket URL to connect to
    /// - `registry`: Type registry for deserializing component data
    /// - `mode`: Display mode (Widget or Embedded). Defaults to Widget.
    #[component]
    pub fn DevTools(
        ws_url: &'static str,
        registry: ComponentTypeRegistry,
        #[prop(optional)] mode: DevToolsMode,
    ) -> impl IntoView {
        // Connection + debug state
        let (last_incoming, set_last_incoming) = signal(String::new());
        let (last_error, set_last_error) = signal(Option::<String>::None);
        let (message_expanded, set_message_expanded) = signal(false);
        let (message_flash, set_message_flash) = signal(false);

        // Widget state (for floating mode)
        let (widget_expanded, set_widget_expanded) = signal(false);

        // Live entity/component view built from incoming SyncBatch items.
        let entities = RwSignal::new(HashMap::<u64, HashMap<String, JsonValue>>::new());

        // Client-side subscription tracking so we can render and cancel them.
        let next_subscription_id = RwSignal::new(0_u64);
        let subscriptions = RwSignal::new(Vec::<SubscriptionRequest>::new());
        let selected_entity = RwSignal::new(None::<u64>);
        let auto_subscription_id = RwSignal::new(None::<u64>);


        // Use EventworkBincodeCodec to receive NetworkPacket directly
        // This gives us better error handling and debugging
        let UseWebSocketReturn { ready_state, message: raw_message, send: raw_send, open, close, .. } =
            use_websocket_with_options::<
                NetworkPacket,
                NetworkPacket,
                EventworkBincodeCodec,
                (),
                DummyEncoder,
            >(
                ws_url,
                UseWebSocketOptions::default()
                    .immediate(true)  // Auto-connect immediately
                    .on_open(move |_| {
                        console::log_1(&"[DevTools] WebSocket opened!".into());
                        set_last_error.set(None);
                    })
                    .on_error(move |e| {
                        console::error_1(&format!("[DevTools] WebSocket error: {e:?}").into());
                        set_last_error.set(Some(format!("{e:?}")));
                    }),
            );

        // Unwrap NetworkPacket and deserialize to SyncServerMessage
        let message = Signal::derive(move || {
            raw_message.with(|packet_opt| {
                console::log_1(&format!("[DevTools] raw_message signal fired, packet present: {}", packet_opt.is_some()).into());

                packet_opt.as_ref().and_then(|packet| {
                    console::log_1(&format!("[DevTools] Received NetworkPacket: type_name={}, schema_hash={}, data_len={}", packet.type_name, packet.schema_hash, packet.data.len()).into());

                    // Use bincode v2 serde API with standard config
                    match bincode::serde::decode_from_slice(&packet.data, bincode::config::standard()) {
                        Ok((msg, _)) => {
                            console::log_1(&format!("[DevTools] Successfully deserialized SyncServerMessage").into());
                            Some(msg)
                        },
                        Err(e) => {
                            console::error_1(&format!("[DevTools] Failed to deserialize SyncServerMessage from NetworkPacket: {:?}", e).into());
                            console::error_1(&format!("[DevTools] NetworkPacket: type_name={}, schema_hash={}, data_len={}", packet.type_name, packet.schema_hash, packet.data.len()).into());
                            set_last_error.set(Some(format!("Deserialization error: {:?}", e)));
                            None
                        }
                    }
                })
            })
        });

        // Wrap send to serialize SyncClientMessage into NetworkPacket
        let send = move |msg: &SyncClientMessage| {
            let packet = NetworkPacket {
                type_name: std::any::type_name::<SyncClientMessage>().to_string(),
                schema_hash: 0, // TODO: compute proper schema hash
                data: bincode::serde::encode_to_vec(msg, bincode::config::standard()).unwrap(),
            };
            raw_send(&packet);
        };

        // General sync hook powered by the WebSocket transport.
        let sync = {
            let registry_clone = registry.clone();
            let s = use_sync(move |msg: SyncClientMessage| {
                send(&msg);
            }, registry_clone);
            RwSignal::new(s)
        };

        // Provide the SyncClient via context so other components can use it
        provide_context(sync.get_untracked().client().clone());

        // React to incoming server messages: update mutation state and
        // maintain a simple entity/component projection.
        {
            let sync = sync;
            let entities = entities;
            let set_last_incoming = set_last_incoming;
            let set_message_flash = set_message_flash;
            let registry = registry.clone();
            Effect::new(move |_| {
                message.with(|msg| {
                    if let Some(msg) = msg {
                        if let Ok(json) = serde_json::to_string_pretty(msg) {
                            set_last_incoming.set(json);
                        }

                        // Trigger flash animation
                        set_message_flash.set(true);
                        set_timeout(move || {
                            set_message_flash.set(false);
                        }, std::time::Duration::from_millis(300));

                        sync.get().handle_server_message(msg);
                        if let SyncServerMessage::SyncBatch(batch) = msg {
                            entities.update(|map| {
                                for item in &batch.items {
                                    match item {
                                        SyncItem::Snapshot { entity, component_type, value, .. }
                                        | SyncItem::Update { entity, component_type, value, .. } => {
                                            // Use the type registry to deserialize component data
                                            match registry.deserialize_by_name(component_type, value) {
                                                Ok(json_value) => {
                                                    map.entry(entity.bits)
                                                        .or_default()
                                                        .insert(component_type.clone(), json_value);
                                                }
                                                Err(e) => {
                                                    console::error_1(&format!("[DevTools] Failed to deserialize component '{}': {}", component_type, e).into());
                                                }
                                            }
                                        }
                                        SyncItem::ComponentRemoved { entity, component_type, .. } => {
                                            if let Some(entry) = map.get_mut(&entity.bits) {
                                                entry.remove(component_type);
                                                if entry.is_empty() {
                                                    map.remove(&entity.bits);
                                                }
                                            }
                                        }
                                        SyncItem::EntityRemoved { entity, .. } => {
                                            map.remove(&entity.bits);
                                        }
                                    }
                                }
                            });
                        }
                    }
                });
            });
        }
        // Automatically subscribe to all components once the WebSocket is open.
        {
            let sync = sync;
            let entities = entities;
            let subscriptions = subscriptions;
            let selected_entity = selected_entity;
            let auto_subscription_id = auto_subscription_id;
            let next_subscription_id = next_subscription_id;
            Effect::new(move |_| {
                let state = ready_state.get();
                if state == ConnectionReadyState::Open && auto_subscription_id.get().is_none() {
                    let id = next_subscription_id.get() + 1;
                    next_subscription_id.set(id);
                    let req = SubscriptionRequest { subscription_id: id, component_type: "*".to_string(), entity: None };
                    sync.get().send_raw(SyncClientMessage::Subscription(req.clone()));
                    auto_subscription_id.set(Some(id));
                    subscriptions.update(|subs| subs.push(req));
                } else if state != ConnectionReadyState::Open && auto_subscription_id.get().is_some() {
                    auto_subscription_id.set(None);
                    subscriptions.update(|subs| subs.clear());
                    entities.set(HashMap::new());
                    selected_entity.set(None);
                }
            });
        }


        let connection_label = move || match ready_state.get() {
            ConnectionReadyState::Connecting => "Connecting",
            ConnectionReadyState::Open => "Open",
            ConnectionReadyState::Closing => "Closing",
            ConnectionReadyState::Closed => "Closed",
        };

        // View mode: true = tree view, false = flat view
        let tree_view_mode = RwSignal::new(true);

        // Track which entities are expanded in tree view (entity_id -> is_expanded)
        // Default to expanded for all entities
        let expanded_entities = RwSignal::new(HashMap::<u64, bool>::new());

        let sorted_entities = move || {
            let mut v: Vec<_> = entities.get().into_iter().collect();
            v.sort_by_key(|(id, _)| *id);
            v
        };

        // Build tree structure from ParentEntity and ChildEntities components
        let entity_tree = move || {
            let all_entities = entities.get();
            let mut roots = Vec::new();
            let mut children_map: HashMap<u64, Vec<u64>> = HashMap::new();

            // First pass: collect all parent-child relationships
            for (entity_id, components) in &all_entities {
                // Check if this entity has a ParentEntity component
                if let Some(JsonValue::Object(parent_comp)) = components.get("ParentEntity") {
                    if let Some(JsonValue::Number(parent_bits)) = parent_comp.get("parent_bits") {
                        if let Some(parent_id) = parent_bits.as_u64() {
                            children_map.entry(parent_id).or_default().push(*entity_id);
                        }
                    }
                }
            }

            // Second pass: find root entities (those without ParentEntity)
            for (entity_id, components) in &all_entities {
                if !components.contains_key("ParentEntity") {
                    roots.push(*entity_id);
                }
            }

            // Sort roots and children for consistent ordering
            roots.sort();
            for children in children_map.values_mut() {
                children.sort();
            }

            (roots, children_map)
        };

        let selected = move || {
            selected_entity
                .get()
                .and_then(|id| entities.get().get(&id).cloned().map(|components| (id, components)))
        };

        // Render based on mode
        match mode {
            DevToolsMode::Embedded => {
                // Full-page embedded view with fixed viewport height
                view! {
            <div class="fixed inset-0 w-full h-full bg-gradient-to-b from-slate-950 via-slate-900 to-slate-950 text-slate-50 flex flex-col overflow-hidden">
                <header class="border-b border-white/5 bg-slate-900/80 backdrop-blur-sm shadow-sm px-4 py-3 flex items-center justify-between flex-shrink-0">
                    <div>
                        <h1 class="text-lg font-semibold tracking-tight">"Eventwork DevTools"</h1>
                        <p class="text-xs text-slate-400">"Realtime ECS inspector & mutation console"</p>
                    </div>
                    <div class="flex items-center gap-3 text-xs">
                        <span class="px-2 py-1 rounded-full border border-slate-700 bg-slate-900">
                            {move || format!("{} · {}", connection_label(), ws_url)}
                        </span>
                        <button
                            class="px-3 py-1 rounded bg-emerald-500 text-slate-950 font-medium disabled:opacity-50"
                            on:click=move |_| open()
                            disabled=move || ready_state.get() == ConnectionReadyState::Open
                        >"Connect"</button>
                        <button
                            class="px-3 py-1 rounded bg-slate-700 text-slate-50 disabled:opacity-50"
                            on:click=move |_| close()
                            disabled=move || ready_state.get() != ConnectionReadyState::Open
                        >"Disconnect"</button>
                    </div>
                </header>

                <main class="flex-1 overflow-hidden grid grid-cols-12 gap-4 p-4 min-h-0">
                    <section class="col-span-3 flex flex-col gap-3 min-h-0">
                        <div class="rounded-2xl border border-white/5 bg-slate-900/70 backdrop-blur-sm shadow-lg shadow-black/40 p-3 flex flex-col min-h-0 h-full">
                            <div class="flex items-center justify-between mb-2 flex-shrink-0">
                                <h2 class="text-sm font-semibold text-slate-100">"World"</h2>
                                <div class="flex items-center gap-2">
                                    <button
                                        class="px-2 py-1 text-[10px] rounded border border-white/10 bg-slate-800/50 hover:bg-slate-700/50 transition-colors"
                                        on:click=move |_| tree_view_mode.update(|mode| *mode = !*mode)
                                    >
                                        {move || if tree_view_mode.get() { "Tree View" } else { "Flat View" }}
                                    </button>
                                    <Show when=move || tree_view_mode.get()>
                                        <button
                                            class="px-2 py-1 text-[10px] rounded border border-white/10 bg-slate-800/50 hover:bg-slate-700/50 transition-colors"
                                            on:click=move |_| {
                                                let (_, children_map) = entity_tree();

                                                // Check if all entities with children are expanded
                                                let all_expanded = expanded_entities.with(|map| {
                                                    children_map.keys().all(|id| map.get(id).copied().unwrap_or(true))
                                                });

                                                // Toggle all
                                                expanded_entities.update(|map| {
                                                    for entity_id in children_map.keys() {
                                                        map.insert(*entity_id, !all_expanded);
                                                    }
                                                });
                                            }
                                        >
                                            {move || {
                                                let (_, children_map) = entity_tree();
                                                let all_expanded = expanded_entities.with(|map| {
                                                    children_map.keys().all(|id| map.get(id).copied().unwrap_or(true))
                                                });
                                                if all_expanded { "Collapse All" } else { "Expand All" }
                                            }}
                                        </button>
                                    </Show>
                                    <span class="text-[11px] text-slate-400">
                                        {move || format!("{} entities", entities.get().len())}
                                    </span>
                                </div>
                            </div>
                            <div class="flex-1 overflow-y-auto space-y-1 text-xs min-h-0">
                                <Show
                                    when=move || !entities.get().is_empty()
                                    fallback=move || view! {
                                        <div class="text-[11px] text-slate-500">
                                            "No entities yet. Connect a Bevy app with EventworkSyncPlugin."
                                        </div>
                                    }
                                >
                                    <Show
                                        when=move || tree_view_mode.get()
                                        fallback=move || view! {
                                            // Flat view
                                            <For
                                                each=sorted_entities
                                                key=|(id, _)| *id
                                                children=move |(id, components): (u64, HashMap<String, JsonValue>)| {
                                                    let label = entity_label(id, &components);
                                                    let selected_entity = selected_entity;
                                                    view! {
                                                        <button
                                                            class=move || {
                                                        let is_selected = selected_entity.get() == Some(id);
                                                        let base = "w-full text-left px-2 py-1.5 rounded-md border transition-colors";
                                                        if is_selected {
                                                            format!("{base} bg-indigo-600/80 border-indigo-500 text-slate-50")
                                                        } else {
                                                            format!("{base} bg-slate-900/40 border-slate-800 text-slate-300 hover:bg-slate-800/70")
                                                        }
                                                    }
                                                    on:click=move |_| selected_entity.set(Some(id))
                                                >
                                                    <div class="flex items-center justify-between gap-2">
                                                        <span class="truncate text-[11px] font-medium">{label}</span>
                                                        <span class="text-[10px] text-slate-400">
                                                            {format!("{} comps", components.len())}
                                                        </span>
                                                    </div>
                                                    <div class="text-[10px] text-slate-500 font-mono mt-0.5">
                                                        "#"{id}
                                                    </div>
                                                </button>
                                            }
                                        }
                                    />
                                        }
                                    >
                                        // Tree view - render entities hierarchically with accordion
                                        {move || {
                                            let (roots, children_map) = entity_tree();
                                            let all_entities = entities.get();
                                            let expanded = expanded_entities;

                                            // Recursive function to render entity and its children
                                            fn render_entity_tree(
                                                entity_id: u64,
                                                components: &HashMap<String, JsonValue>,
                                                children_map: &HashMap<u64, Vec<u64>>,
                                                all_entities: &HashMap<u64, HashMap<String, JsonValue>>,
                                                selected_entity: RwSignal<Option<u64>>,
                                                expanded_entities: RwSignal<HashMap<u64, bool>>,
                                                depth: usize,
                                            ) -> Vec<AnyView> {
                                                let mut views = Vec::new();
                                                let label = entity_label(entity_id, components);
                                                let has_children = children_map.contains_key(&entity_id);

                                                // Check if this entity is expanded (default to true)
                                                let is_expanded = expanded_entities.with(|map| {
                                                    map.get(&entity_id).copied().unwrap_or(true)
                                                });

                                                // Render this entity with expand/collapse icon if it has children
                                                let entity_view = view! {
                                                    <div class="flex items-center gap-1">
                                                        {if has_children {
                                                            view! {
                                                                <button
                                                                    class="flex-shrink-0 w-4 h-4 flex items-center justify-center text-slate-400 hover:text-slate-200 transition-colors"
                                                                    on:click=move |e| {
                                                                        e.stop_propagation();
                                                                        expanded_entities.update(|map| {
                                                                            let current = map.get(&entity_id).copied().unwrap_or(true);
                                                                            map.insert(entity_id, !current);
                                                                        });
                                                                    }
                                                                >
                                                                    <span class="text-[10px]">
                                                                        {move || if expanded_entities.with(|map| map.get(&entity_id).copied().unwrap_or(true)) { "▼" } else { "▶" }}
                                                                    </span>
                                                                </button>
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <div class="w-4"></div>
                                                            }.into_any()
                                                        }}
                                                        <button
                                                            class=move || {
                                                                let is_selected = selected_entity.get() == Some(entity_id);
                                                                let base = "flex-1 text-left px-2 py-1.5 rounded-md border transition-colors";
                                                                if is_selected {
                                                                    format!("{base} bg-indigo-600/80 border-indigo-500 text-slate-50")
                                                                } else {
                                                                    format!("{base} bg-slate-900/40 border-slate-800 text-slate-300 hover:bg-slate-800/70")
                                                                }
                                                            }
                                                            on:click=move |_| selected_entity.set(Some(entity_id))
                                                        >
                                                            <div class="flex items-center justify-between gap-2">
                                                                <span class="truncate text-[11px] font-medium">{label}</span>
                                                                <span class="text-[10px] text-slate-400">
                                                                    {format!("{} comps", components.len())}
                                                                </span>
                                                            </div>
                                                            <div class="text-[10px] text-slate-500 font-mono mt-0.5">
                                                                "#"{entity_id}
                                                            </div>
                                                        </button>
                                                    </div>
                                                }.into_any();
                                                views.push(entity_view);

                                                // Render children recursively only if expanded
                                                if is_expanded {
                                                    if let Some(children) = children_map.get(&entity_id) {
                                                        for child_id in children {
                                                            if let Some(child_components) = all_entities.get(child_id) {
                                                                // Wrap children in a container with left margin for indentation
                                                                let child_views = view! {
                                                                    <div class="ml-4">
                                                                        {render_entity_tree(
                                                                            *child_id,
                                                                            child_components,
                                                                            children_map,
                                                                            all_entities,
                                                                            selected_entity,
                                                                            expanded_entities,
                                                                            depth + 1,
                                                                        )}
                                                                    </div>
                                                                }.into_any();
                                                                views.push(child_views);
                                                            }
                                                        }
                                                    }
                                                }

                                                views
                                            }

                                            // Render all root entities and their trees
                                            let mut all_views = Vec::new();
                                            for root_id in roots {
                                                if let Some(root_components) = all_entities.get(&root_id) {
                                                    let tree_views = render_entity_tree(
                                                        root_id,
                                                        root_components,
                                                        &children_map,
                                                        &all_entities,
                                                        selected_entity,
                                                        expanded,
                                                        0,
                                                    );
                                                    all_views.extend(tree_views);
                                                }
                                            }

                                            all_views
                                        }}
                                    </Show>
                                </Show>
                            </div>
                        </div>
                    </section>

                    <section class="col-span-6 rounded-2xl border border-white/5 bg-slate-900/70 backdrop-blur-sm shadow-lg shadow-black/40 p-4 flex flex-col min-h-0">
                        <h2 class="text-sm font-semibold text-slate-100 mb-2 flex-shrink-0">"Inspector"</h2>
                        <div class="flex-1 overflow-y-auto min-h-0">
                            <Show
                                when=move || selected().is_some()
                                fallback=move || view! {
                                    <div class="text-[11px] text-slate-500">
                                        "Select an entity from the left to inspect and edit its components."
                                    </div>
                                }
                            >
                                {move || {
                                    let Some((id, components)) = selected() else {
                                        return ().into_view().into_any();
                                    };

                                    let label = entity_label(id, &components);
                                    view! {
                                        <div class="flex flex-col gap-3 text-xs">
                                            <div class="flex items-center justify-between">
                                                <div>
                                                    <div class="text-[11px] uppercase tracking-wide text-slate-500">"Entity"</div>
                                                    <div class="text-sm font-semibold text-slate-50">{label}</div>
                                                </div>
                                                <div class="text-[10px] text-slate-500 font-mono">
                                                    "#"{id}
                                                </div>
                                            </div>
                                            <div class="border-t border-slate-800 pt-3 space-y-3">
                                            <For
                                                each=move || {
                                                    let mut v: Vec<(String, JsonValue)> = components
                                                        .iter()
                                                        .map(|(ty, value)| (ty.clone(), value.clone()))
                                                        .collect();
                                                    v.sort_by_key(|(ty, _)| ty.clone());
                                                    v
                                                }
                                                key=|(ty, _)| ty.clone()
                                                children=move |(ty, value): (String, JsonValue)| {
                                                    let entities_for = entities;
                                                    let sync_for = sync;
                                                    let ty_for = ty.clone();
                                                    let id_for = id;
                                                    let body: AnyView = match value {
                                                        JsonValue::Object(_) => {
                                                            component_editor(id_for, ty_for.clone(), entities_for, sync_for)
                                                                .into_view()
                                                                .into_any()
                                                        }
                                                        other => {
                                                            let json = serde_json::to_string_pretty(&other).unwrap_or_default();
                                                            view! {
                                                                <pre class="mt-1 bg-slate-950/60 border border-slate-800 rounded p-1 font-mono text-[10px] whitespace-pre-wrap break-all">
                                                                    {json}
                                                                </pre>
                                                            }.into_any()
                                                        }
                                                    };
                                                    view! {
                                                        <div class="border border-slate-800 rounded-md p-2 space-y-1">
                                                            <div class="flex items-center justify-between">
                                                                <div class="text-[11px] text-indigo-300 font-medium">{ty.clone()}</div>
                                                            </div>
                                                            {body}
                                                        </div>
                                                    }
                                                }
                                            />
                                            </div>
                                        </div>
                                    }.into_any()
                                }}
                            </Show>
                        </div>
                    </section>

                    <section class="col-span-3 flex flex-col gap-3 min-h-0">
                        <div class="rounded-2xl border border-white/5 bg-slate-900/70 backdrop-blur-sm shadow-lg shadow-black/40 p-3 text-xs space-y-1 flex-shrink-0">
                            <div class="flex items-center justify-between">
                                <span class="font-semibold">"Status"</span>
                                <span class="text-slate-400">{move || format!("{:?}", ready_state.get())}</span>
                            </div>
                            <div class="mt-1 text-[11px] text-slate-400">
                                {move || {
                                    if let Some(id) = auto_subscription_id.get() {
                                        format!("Wildcard subscription · #{}", id)
                                    } else {
                                        "No active subscriptions".to_string()
                                    }
                                }}
                            </div>
                            <div class="mt-1 text-[11px] text-slate-400">
                                {move || format!("Entities mirrored · {}", entities.get().len())}
                            </div>
                            <Show
                                when=move || last_error.get().is_some()
                                fallback=|| view! { <></> }
                            >
                                <div class="mt-1 text-red-400">{move || last_error.get().unwrap_or_default()}</div>
                            </Show>
                        </div>
                        <div class="rounded-2xl border border-white/5 bg-slate-900/70 backdrop-blur-sm shadow-lg shadow-black/40 p-3 flex flex-col min-h-0 flex-1">
                            <button
                                class="flex items-center justify-between w-full text-left group flex-shrink-0"
                                on:click=move |_| set_message_expanded.update(|v| *v = !*v)
                            >
                                <div class="flex items-center gap-2">
                                    <h2 class="text-sm font-semibold text-slate-100">"Server Messages"</h2>
                                    <div
                                        class=move || {
                                            let base = "w-2 h-2 rounded-full transition-all duration-300";
                                            if message_flash.get() {
                                                format!("{base} bg-green-400 shadow-lg shadow-green-400/50")
                                            } else {
                                                format!("{base} bg-slate-700")
                                            }
                                        }
                                    ></div>
                                </div>
                                <span class="text-slate-400 text-xs group-hover:text-slate-300 transition-colors">
                                    {move || if message_expanded.get() { "▼" } else { "▶" }}
                                </span>
                            </button>
                            <Show
                                when=move || message_expanded.get()
                                fallback=|| view! { <></> }
                            >
                                <div class="mt-2 flex-1 overflow-y-auto min-h-0 h-full">
                                    <pre class="text-[10px] font-mono bg-slate-950/60 border border-slate-800 rounded p-2 whitespace-pre-wrap break-all h-full">
                                        {move || last_incoming.get()}
                                    </pre>
                                </div>
                            </Show>
                        </div>
                    </section>
                </main>
            </div>
        }.into_any()
            }
            DevToolsMode::Widget => {
                // Floating widget mode
                let open_in_new_tab = move |_| {
                    // Open DevTools in a new tab/window with ?devtools=1 query param
                    if let Some(window) = leptos::web_sys::window() {
                        let _ = window.open_with_url_and_target(
                            "?devtools=1",
                            "_blank"
                        );
                    }
                };

                view! {
                    <div>
                        // Floating widget button (collapsed state)
                        <Show
                            when=move || !widget_expanded.get()
                            fallback=|| view! { <></> }
                        >
                            <button
                                class="fixed bottom-4 left-4 z-50 flex items-center gap-2 px-3 py-2 bg-gradient-to-r from-indigo-600 to-purple-600 text-white rounded-full shadow-lg hover:shadow-xl transition-all duration-200 hover:scale-105 border border-white/20"
                                on:click=move |_| set_widget_expanded.set(true)
                            >
                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"></path>
                                </svg>
                                <span class="text-xs font-semibold">"DevTools"</span>
                                <Show when=move || !entities.get().is_empty()>
                                    <span class="px-1.5 py-0.5 bg-white/20 rounded-full text-[10px] font-bold">
                                        {move || entities.get().len()}
                                    </span>
                                </Show>
                            </button>
                        </Show>

                        // Modal overlay (expanded state)
                        <Show
                            when=move || widget_expanded.get()
                            fallback=|| view! { <></> }
                        >
                            <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
                                <div class="relative w-[95vw] h-[90vh] max-w-[1800px] rounded-2xl shadow-2xl overflow-hidden border border-white/10">
                                    // Render the full DevTools UI inside the modal
                                    // Call DevTools recursively with Embedded mode
                                    <DevTools ws_url=ws_url registry=registry.clone() mode=DevToolsMode::Embedded />

                                    // Action buttons at bottom-left (away from Connect button)
                                    <div class="absolute bottom-4 left-4 z-10 flex gap-2">
                                        <button
                                            class="px-3 py-1.5 bg-slate-800/90 hover:bg-slate-700/90 text-slate-200 rounded-lg text-xs font-medium transition-colors border border-white/10 flex items-center gap-1.5 shadow-lg"
                                            on:click=open_in_new_tab
                                        >
                                            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path>
                                            </svg>
                                            "Open in New Tab"
                                        </button>
                                        <button
                                            class="px-3 py-1.5 bg-slate-800/90 hover:bg-slate-700/90 text-slate-200 rounded-lg text-xs font-medium transition-colors border border-white/10 shadow-lg"
                                            on:click=move |_| set_widget_expanded.set(false)
                                        >
                                            "✕ Close"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </Show>
                    </div>
                }.into_any()
            }
        }
    }
}
