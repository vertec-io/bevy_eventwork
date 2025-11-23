//! Shared types for eventwork_client basic example
//!
//! This crate defines component types that are shared between the Bevy server
//! and the Leptos web client.

use serde::{Deserialize, Serialize};
use eventwork_client::impl_sync_component;

#[cfg(feature = "server")]
use bevy::prelude::*;

#[cfg(feature = "stores")]
use reactive_stores::Store;

/// 2D position component
#[cfg_attr(feature = "server", derive(Component))]
#[cfg_attr(feature = "stores", derive(Store))]
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl_sync_component!(Position);

/// 2D velocity component
#[cfg_attr(feature = "server", derive(Component))]
#[cfg_attr(feature = "stores", derive(Store))]
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl_sync_component!(Velocity);

/// Name component
#[cfg_attr(feature = "server", derive(Component))]
#[cfg_attr(feature = "stores", derive(Store))]
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct EntityName {
    pub name: String,
}

impl_sync_component!(EntityName);

