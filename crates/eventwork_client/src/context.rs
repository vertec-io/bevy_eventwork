use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use leptos::prelude::*;
use leptos_use::core::ConnectionReadyState;

use crate::error::SyncError;
use crate::registry::ClientRegistry;
use crate::traits::SyncComponent;
use eventwork_sync::{SerializableEntity, SubscriptionRequest, UnsubscribeRequest, SyncClientMessage};

/// Connection control interface exposed to components.
///
/// This allows components to manually control the WebSocket connection.
#[derive(Clone)]
pub struct SyncConnection {
    /// Current connection state
    pub ready_state: Signal<ConnectionReadyState>,
    /// Open the WebSocket connection
    pub open: Arc<dyn Fn() + Send + Sync>,
    /// Close the WebSocket connection
    pub close: Arc<dyn Fn() + Send + Sync>,
}

/// Context providing access to the sync client.
///
/// This context is provided by `SyncProvider` and consumed by hooks like
/// `use_sync_component`. It manages subscription lifecycle, caching, and
/// message routing.
#[derive(Clone)]
pub struct SyncContext {
    /// Current connection state
    pub ready_state: Signal<ConnectionReadyState>,
    /// Last error that occurred
    pub last_error: Signal<Option<SyncError>>,
    /// Function to send messages to the server
    send: Arc<dyn Fn(&[u8]) + Send + Sync>,
    /// Function to open the connection
    open: Arc<dyn Fn() + Send + Sync>,
    /// Function to close the connection
    close: Arc<dyn Fn() + Send + Sync>,
    /// Type registry for deserialization
    registry: Arc<ClientRegistry>,
    /// Cache of signals for each (TypeId, params) pair
    /// Uses Weak references to allow garbage collection
    signal_cache: Arc<Mutex<HashMap<(TypeId, String), Weak<dyn Any + Send + Sync>>>>,
    /// Subscription tracking: component_type -> (subscription_id, ref_count)
    subscriptions: Arc<Mutex<HashMap<String, (u64, usize)>>>,
    /// Next subscription ID
    next_subscription_id: Arc<Mutex<u64>>,
    /// Raw component data storage: (entity_id, component_name) -> raw bytes
    /// This is the central storage that handle_sync_item updates
    /// Effects in subscribe_component watch this and deserialize to typed signals
    pub(crate) component_data: RwSignal<HashMap<(u64, String), Vec<u8>>>,
}

impl SyncContext {
    /// Create a new SyncContext.
    ///
    /// This is typically called by `SyncProvider`, not by user code.
    pub fn new(
        ready_state: Signal<ConnectionReadyState>,
        last_error: Signal<Option<SyncError>>,
        send: Arc<dyn Fn(&[u8]) + Send + Sync>,
        open: Arc<dyn Fn() + Send + Sync>,
        close: Arc<dyn Fn() + Send + Sync>,
        registry: Arc<ClientRegistry>,
    ) -> Self {
        Self {
            ready_state,
            last_error,
            send,
            open,
            close,
            registry,
            signal_cache: Arc::new(Mutex::new(HashMap::new())),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            next_subscription_id: Arc::new(Mutex::new(0)),
            component_data: RwSignal::new(HashMap::new()),
        }
    }

    /// Get connection control interface.
    pub fn connection(&self) -> SyncConnection {
        SyncConnection {
            ready_state: self.ready_state,
            open: self.open.clone(),
            close: self.close.clone(),
        }
    }

    /// Subscribe to a component type.
    ///
    /// This returns a signal containing a HashMap of entity_id -> component.
    /// Multiple calls with the same type will return the same signal (deduplication).
    ///
    /// The subscription is automatically managed:
    /// - Sends SubscriptionRequest when first component subscribes
    /// - Returns cached signal for subsequent subscriptions
    /// - Sends UnsubscribeRequest when last component unsubscribes
    pub fn subscribe_component<T: SyncComponent + Clone + Default>(
        &self,
    ) -> ReadSignal<HashMap<u64, T>> {
        let component_name = T::component_name();
        let type_id = TypeId::of::<T>();
        let cache_key = (type_id, String::new()); // Empty string for no params

        // Try to get existing signal from cache
        {
            let cache = self.signal_cache.lock().unwrap();
            if let Some(weak_signal) = cache.get(&cache_key) {
                if let Some(strong_signal) = weak_signal.upgrade() {
                    if let Some(signal) = strong_signal.downcast_ref::<Arc<RwSignal<HashMap<u64, T>>>>() {
                        // Increment ref count (but don't send subscription request - already subscribed)
                        self.increment_subscription(component_name);

                        // Set up cleanup on unmount
                        let ctx = self.clone();
                        let component_name_owned = component_name.to_string();
                        on_cleanup(move || {
                            if let Some(subscription_id) = ctx.decrement_subscription(&component_name_owned) {
                                ctx.send_unsubscribe_request(subscription_id);
                            }
                        });

                        return signal.read_only();
                    }
                }
            }
        }

        // Create new signal
        let signal = RwSignal::new(HashMap::new());
        let signal_arc = Arc::new(signal);

        // Cache the signal
        {
            let mut cache = self.signal_cache.lock().unwrap();
            cache.insert(
                cache_key,
                Arc::downgrade(&(signal_arc.clone() as Arc<dyn Any + Send + Sync>)),
            );
        }

        // Increment ref count and send subscription request if this is the first subscription
        let is_first = self.increment_subscription(component_name);
        if is_first {
            // Set up an Effect to send the subscription request when the WebSocket is open
            let ctx = self.clone();
            let component_name_owned = component_name.to_string();
            let ready_state = self.ready_state;

            Effect::new(move |_| {
                if ready_state.get() == ConnectionReadyState::Open {
                    #[cfg(target_arch = "wasm32")]
                    leptos::logging::log!(
                        "[SyncContext] WebSocket is open, sending subscription request for '{}'",
                        component_name_owned
                    );

                    ctx.send_subscription_request(&component_name_owned, None);
                }
            });
        }

        // Set up Effect to watch component_data and deserialize to typed signal
        // This is the Meteorite pattern: raw bytes -> Effect -> typed signal
        let component_data = self.component_data;
        let registry = self.registry.clone();
        let component_name_str = component_name.to_string();
        let signal_clone = signal;

        Effect::new(move |_| {
            let data_map = component_data.get();
            let mut typed_map = HashMap::new();

            #[cfg(target_arch = "wasm32")]
            leptos::logging::log!(
                "[SyncContext] Effect triggered for component '{}', data_map has {} entries",
                component_name_str,
                data_map.len()
            );

            // Iterate through all entities and deserialize components of type T
            for ((entity_id, comp_name), bytes) in data_map.iter() {
                if comp_name == &component_name_str {
                    #[cfg(target_arch = "wasm32")]
                    leptos::logging::log!(
                        "[SyncContext] Found matching component '{}' for entity {}, {} bytes",
                        comp_name,
                        entity_id,
                        bytes.len()
                    );

                    // Deserialize the component
                    match registry.deserialize::<T>(comp_name, bytes) {
                        Ok(component) => {
                            #[cfg(target_arch = "wasm32")]
                            leptos::logging::log!(
                                "[SyncContext] Successfully deserialized {} for entity {}",
                                comp_name,
                                entity_id
                            );
                            typed_map.insert(*entity_id, component);
                        }
                        Err(err) => {
                            #[cfg(target_arch = "wasm32")]
                            leptos::logging::warn!(
                                "[SyncContext] Failed to deserialize {} for entity {}: {:?}",
                                comp_name,
                                entity_id,
                                err
                            );
                        }
                    }
                }
            }

            #[cfg(target_arch = "wasm32")]
            leptos::logging::log!(
                "[SyncContext] Setting signal for '{}' with {} entities",
                component_name_str,
                typed_map.len()
            );

            // Update the typed signal
            signal_clone.set(typed_map);
        });

        // Set up cleanup on unmount
        let ctx = self.clone();
        let component_name_owned = component_name.to_string();
        on_cleanup(move || {
            if let Some(subscription_id) = ctx.decrement_subscription(&component_name_owned) {
                ctx.send_unsubscribe_request(subscription_id);
            }
        });

        signal_clone.read_only()
    }

    /// Increment subscription ref count. Returns true if this is the first subscription.
    fn increment_subscription(&self, component_name: &str) -> bool {
        let mut subs = self.subscriptions.lock().unwrap();
        if let Some((_, ref_count)) = subs.get_mut(component_name) {
            *ref_count += 1;
            false // Not the first subscription
        } else {
            // First subscription - allocate a new subscription ID
            let subscription_id = {
                let mut id = self.next_subscription_id.lock().unwrap();
                let current = *id;
                *id += 1;
                current
            };
            subs.insert(component_name.to_string(), (subscription_id, 1));
            true // First subscription
        }
    }

    /// Decrement subscription ref count. Returns Some(subscription_id) if this was the last subscription.
    fn decrement_subscription(&self, component_name: &str) -> Option<u64> {
        let mut subs = self.subscriptions.lock().unwrap();
        if let Some((subscription_id, ref_count)) = subs.get_mut(component_name) {
            *ref_count -= 1;
            if *ref_count == 0 {
                let id = *subscription_id;
                subs.remove(component_name);
                return Some(id);
            }
        }
        None
    }

    /// Send a subscription request to the server.
    fn send_subscription_request(&self, component_name: &str, entity: Option<SerializableEntity>) {
        // Get the subscription ID for this component type
        let subscription_id = {
            let subs = self.subscriptions.lock().unwrap();
            subs.get(component_name).map(|(id, _)| *id).unwrap_or(0)
        };

        let request = SubscriptionRequest {
            subscription_id,
            component_type: component_name.to_string(),
            entity,
        };

        // Wrap in SyncClientMessage and serialize
        let message = SyncClientMessage::Subscription(request);
        if let Ok(bytes) = bincode::serde::encode_to_vec(&message, bincode::config::standard()) {
            (self.send)(&bytes);
        }
    }

    /// Send an unsubscribe request to the server.
    fn send_unsubscribe_request(&self, subscription_id: u64) {
        let request = UnsubscribeRequest {
            subscription_id,
        };

        // Wrap in SyncClientMessage and serialize
        let message = SyncClientMessage::Unsubscribe(request);
        if let Ok(bytes) = bincode::serde::encode_to_vec(&message, bincode::config::standard()) {
            (self.send)(&bytes);
        }
    }

    /// Handle incoming component update from the server.
    ///
    /// This deserializes the component data and updates the appropriate signal.
    pub fn handle_component_update<T: SyncComponent + Clone>(
        &self,
        entity_id: u64,
        data: &[u8],
    ) -> Result<(), SyncError> {
        let component_name = T::component_name();
        let component: T = self.registry.deserialize(component_name, data)?;

        // Find the signal in the cache and update it
        let type_id = TypeId::of::<T>();
        let cache_key = (type_id, String::new());

        let cache = self.signal_cache.lock().unwrap();
        if let Some(weak_signal) = cache.get(&cache_key) {
            if let Some(strong_signal) = weak_signal.upgrade() {
                if let Some(signal_arc) = strong_signal.downcast_ref::<Arc<RwSignal<HashMap<u64, T>>>>() {
                    signal_arc.update(|map| {
                        map.insert(entity_id, component);
                    });
                }
            }
        }

        Ok(())
    }

    /// Handle component removal from the server.
    pub fn handle_component_removed<T: SyncComponent>(&self, entity_id: u64) {
        let type_id = TypeId::of::<T>();
        let cache_key = (type_id, String::new());

        let cache = self.signal_cache.lock().unwrap();
        if let Some(weak_signal) = cache.get(&cache_key) {
            if let Some(strong_signal) = weak_signal.upgrade() {
                if let Some(signal_arc) = strong_signal.downcast_ref::<Arc<RwSignal<HashMap<u64, T>>>>() {
                    signal_arc.update(|map| {
                        map.remove(&entity_id);
                    });
                }
            }
        }
    }
}

