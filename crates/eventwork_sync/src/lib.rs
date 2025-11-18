//! eventwork_sync
//!
//! Reflection-driven synchronization middleware between a Bevy ECS server and
//! arbitrary clients over eventwork.
//!
//! This crate is intentionally generic and does not contain any application-
//! specific logic (no robotics/meteorite assumptions). It exposes:
//!
//! - [`EventworkSyncPlugin`]: wires core resources and systems.
//! - [`AppEventworkSyncExt`]: `sync_component::<T>()` for opt-in component sync.
//! - Wire-level message types for subscriptions, updates, mutations, and
//!   database-backed queries.
//! - [`MutationAuthorizer`] / [`MutationAuthorizerResource`]: pluggable
//!   authorization policies for client-driven mutations, plus a built-in
//!   [`ServerOnlyMutationAuthorizer`] for "server-only" mutation deployments.

mod messages;
#[cfg(feature = "runtime")]
mod registry;
#[cfg(feature = "runtime")]
mod subscription;
#[cfg(feature = "runtime")]
mod systems;

// Client-side type registry (not behind runtime feature - available for all clients)
pub mod client_registry;

pub use messages::*;
#[cfg(feature = "runtime")]
pub use registry::*;
#[cfg(feature = "runtime")]
pub use subscription::*;

#[cfg(feature = "runtime")]
use bevy::prelude::*;
#[cfg(feature = "runtime")]
use eventwork::managers::NetworkProvider;

/// Top-level plugin that adds sync resources, registers network messages, and
/// installs core systems.
#[cfg(feature = "runtime")]
#[derive(Debug, Clone)]
pub struct EventworkSyncPlugin<NP: NetworkProvider> {
    _marker: std::marker::PhantomData<NP>,
}

#[cfg(feature = "runtime")]
impl<NP: NetworkProvider> Default for EventworkSyncPlugin<NP> {
    fn default() -> Self {
        Self { _marker: std::marker::PhantomData }
    }
}

#[cfg(feature = "runtime")]
impl<NP: NetworkProvider> Plugin for EventworkSyncPlugin<NP> {
    fn build(&self, app: &mut App) {
        info!("[EventworkSyncPlugin::build] CALLED - about to call systems::install");
        systems::install::<NP>(app);
        info!("[EventworkSyncPlugin::build] COMPLETED - systems::install returned");
    }
}

/// Extension trait for registering components for synchronization.
#[cfg(feature = "runtime")]
pub trait AppEventworkSyncExt {
    /// Register a component type `T` to be synchronized with remote clients.
    ///
    /// This is the only call most applications need to make per component type.
    fn sync_component<T>(&mut self, config: Option<ComponentSyncConfig>) -> &mut Self
    where
        T: Component + Reflect + bevy::reflect::GetTypeRegistration + serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + 'static + std::fmt::Debug;
}

#[cfg(feature = "runtime")]
impl AppEventworkSyncExt for App {
    fn sync_component<T>(&mut self, config: Option<ComponentSyncConfig>) -> &mut Self
    where
        T: Component + Reflect + bevy::reflect::GetTypeRegistration + serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + 'static + std::fmt::Debug,
    {
        registry::register_component::<T>(self, config);
        self
    }
}

