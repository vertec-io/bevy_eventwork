//! Shared component types for the eventwork devtools demo.
//!
//! This crate contains component type definitions that are shared between
//! the Bevy server and the Leptos WASM client. These types are:
//! - Serde-compatible for serialization/deserialization
//! - Optionally include Bevy Component + Reflect traits when "server" feature is enabled
//! - Used by both server and client for type-safe communication

use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use bevy::prelude::*;

/// A simple counter component for demonstration purposes.
#[cfg_attr(feature = "server", derive(Component, Reflect))]
#[cfg_attr(feature = "server", reflect(Component))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DemoCounter {
    pub value: i32,
}

/// A flag component with a label and enabled state.
#[cfg_attr(feature = "server", derive(Component, Reflect))]
#[cfg_attr(feature = "server", reflect(Component))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DemoFlag {
    pub label: String,
    pub enabled: bool,
}

