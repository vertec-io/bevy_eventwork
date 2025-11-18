use leptos::prelude::*;
use reactive_graph::traits::{Get, Update};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

use eventwork_sync::{
    MutateComponent,
    MutationResponse,
    MutationStatus,
    SerializableEntity,
    SyncClientMessage,
    SyncServerMessage,
};

pub mod type_registry;
pub use type_registry::{ComponentTypeRegistry, DeserializeError, SerializeError};

/// Client-side state for driving `Mutate` requests and tracking
/// `MutationResponse` messages from an `eventwork_sync` server.
#[derive(Clone)]
pub struct DevtoolsSync {
    send: Arc<dyn Fn(SyncClientMessage) + Send + Sync>,
    next_request_id: RwSignal<u64>,
    mutations: RwSignal<HashMap<u64, MutationState>>,
    registry: ComponentTypeRegistry,
}

/// Per-request mutation state tracked on the client.
#[derive(Clone, Debug)]
pub struct MutationState {
    pub request_id: u64,
    pub status: Option<MutationStatus>,
    pub message: Option<String>,
}

impl MutationState {
    pub fn new_pending(request_id: u64) -> Self {
        Self {
            request_id,
            status: None,
            message: None,
        }
    }
}

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
    let send = Arc::new(send) as Arc<dyn Fn(SyncClientMessage) + Send + Sync>;

    let next_request_id = RwSignal::new(0u64);
    let mutations = RwSignal::new(HashMap::<u64, MutationState>::new());

    DevtoolsSync {
        send,
        next_request_id,
        mutations,
        registry,
    }
}

impl DevtoolsSync {
    /// Send a raw `SyncClientMessage` without any local bookkeeping.
    ///
    /// This is useful for subscription management or other operations
    /// that don't need per-request client-side tracking.
    pub fn send_raw(&self, message: SyncClientMessage) {
        (self.send)(message);
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
        let request_id = self.next_request_id.get() + 1;
        self.next_request_id.set(request_id);

        // Track the pending mutation locally.
        self.mutations.update(|map| {
            map.insert(request_id, MutationState::new_pending(request_id));
        });

        let component_type_str = component_type.into();

        // Debug: log what we're trying to serialize
        leptos::logging::log!(
            "[DevTools] Serializing mutation: type='{}', json={:?}",
            component_type_str, value
        );

        // Use the type registry to serialize JSON → concrete type → bincode bytes
        let value_bytes = match self.registry.serialize_by_name(&component_type_str, &value) {
            Ok(bytes) => {
                leptos::logging::log!(
                    "[DevTools] Serialized to {} bytes: {:?}",
                    bytes.len(), bytes
                );
                bytes
            }
            Err(e) => {
                leptos::logging::error!(
                    "[DevTools] Failed to serialize mutation for '{}': {}",
                    component_type_str, e
                );
                return request_id;
            }
        };

        let msg = SyncClientMessage::Mutate(MutateComponent {
            request_id: Some(request_id),
            entity,
            component_type: component_type_str,
            value: value_bytes,
        });

        (self.send)(msg);

        request_id
    }

    /// Handle a server-side message, updating mutation state when a
    /// `MutationResponse` is observed.
    pub fn handle_server_message(&self, message: &SyncServerMessage) {
        if let SyncServerMessage::MutationResponse(MutationResponse {
            request_id: Some(request_id),
            status,
            message,
        }) = message
        {
            self.mutations.update(|map| {
                map.entry(*request_id)
                    .and_modify(|state| {
                        state.status = Some(status.clone());
                        state.message = message.clone();
                    })
                    .or_insert_with(|| MutationState {
                        request_id: *request_id,
                        status: Some(status.clone()),
                        message: message.clone(),
                    });
            });
        }
    }

    /// Helper to handle a `MutationResponse` directly, for cases where
    /// the transport layer already demultiplexes server messages.
    pub fn handle_mutation_response(&self, response: &MutationResponse) {
        if let Some(request_id) = response.request_id {
            self.mutations.update(|map| {
                map.entry(request_id)
                    .and_modify(|state| {
                        state.status = Some(response.status.clone());
                        state.message = response.message.clone();
                    })
                    .or_insert_with(|| MutationState {
                        request_id,
                        status: Some(response.status.clone()),
                        message: response.message.clone(),
                    });
            });
        }
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

    /// High-level DevTools surface: given a WebSocket URL speaking the
    /// `eventwork_sync` wire protocol, render a modern Tailwind-powered
    /// inspector + mutation console.
    ///
    /// # Parameters
    /// - `ws_url`: WebSocket URL to connect to
    /// - `registry`: Type registry for deserializing component data
    #[component]
    pub fn DevTools(
        ws_url: &'static str,
        registry: ComponentTypeRegistry,
    ) -> impl IntoView {
        // Connection + debug state
        let (last_incoming, set_last_incoming) = signal(String::new());
        let (last_error, set_last_error) = signal(Option::<String>::None);

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
                    .immediate(false)
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

        // React to incoming server messages: update mutation state and
        // maintain a simple entity/component projection.
        {
            let sync = sync;
            let entities = entities;
            let set_last_incoming = set_last_incoming;
            let registry = registry.clone();
            Effect::new(move |_| {
                message.with(|msg| {
                    if let Some(msg) = msg {
                        if let Ok(json) = serde_json::to_string_pretty(msg) {
                            set_last_incoming.set(json);
                        }
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

        let sorted_entities = move || {
            let mut v: Vec<_> = entities.get().into_iter().collect();
            v.sort_by_key(|(id, _)| *id);
            v
        };

        let selected = move || {
            selected_entity
                .get()
                .and_then(|id| entities.get().get(&id).cloned().map(|components| (id, components)))
        };

        view! {
            <div class="min-h-screen w-full bg-gradient-to-b from-slate-950 via-slate-900 to-slate-950 text-slate-50 flex flex-col">
                <header class="border-b border-white/5 bg-slate-900/80 backdrop-blur-sm shadow-sm px-4 py-3 flex items-center justify-between">
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

                <main class="flex-1 overflow-hidden grid grid-cols-12 gap-4 p-4">
                    <section class="col-span-3 flex flex-col gap-3">
                        <div class="rounded-2xl border border-white/5 bg-slate-900/70 backdrop-blur-sm shadow-lg shadow-black/40 p-3 flex flex-col min-h-0">
                            <div class="flex items-center justify-between mb-2">
                                <h2 class="text-sm font-semibold text-slate-100">"World"</h2>
                                <span class="text-[11px] text-slate-400">
                                    {move || format!("{} entities", entities.get().len())}
                                </span>
                            </div>
                            <div class="flex-1 overflow-y-auto space-y-1 text-xs">
                                <Show
                                    when=move || !entities.get().is_empty()
                                    fallback=move || view! {
                                        <div class="text-[11px] text-slate-500">
                                            "No entities yet. Connect a Bevy app with EventworkSyncPlugin."
                                        </div>
                                    }
                                >
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
                                </Show>
                            </div>
                        </div>
                    </section>

                    <section class="col-span-6 rounded-2xl border border-white/5 bg-slate-900/70 backdrop-blur-sm shadow-lg shadow-black/40 p-4 flex flex-col min-w-0">
                        <h2 class="text-sm font-semibold text-slate-100 mb-2">"Inspector"</h2>
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
                    </section>

                    <section class="col-span-3 flex flex-col gap-3">
                        <div class="rounded-2xl border border-white/5 bg-slate-900/70 backdrop-blur-sm shadow-lg shadow-black/40 p-3 text-xs space-y-1">
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
                        <div class="rounded-2xl border border-white/5 bg-slate-900/70 backdrop-blur-sm shadow-lg shadow-black/40 p-3 flex-1 flex flex-col min-w-0">
                            <h2 class="text-sm font-semibold text-slate-100 mb-1">"Last server message"</h2>
                            <pre class="flex-1 overflow-auto text-[10px] font-mono bg-slate-950/60 border border-slate-800 rounded p-2 whitespace-pre-wrap break-all">
                                {move || last_incoming.get()}
                            </pre>
                        </div>
                    </section>
                </main>
            </div>
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use ui::DevTools;
