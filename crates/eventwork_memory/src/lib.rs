//! # Eventwork Memory
//!
//! This crate provides memory leak detection and prevention tools for the eventwork networking library.
//! It helps identify and fix memory leaks in Bevy applications that use eventwork for networking.
//!
//! ## Features
//!
//! - Memory usage monitoring
//! - Connection cleanup
//! - Message queue monitoring
//! - Resource cleanup
//!
//! ## Usage
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use eventwork_memory::NetworkMemoryPlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(NetworkMemoryPlugin)
//!         .run();
//! }
//! ```

mod memory_diagnostic;
mod connection_cleanup;
mod message_cleanup;
mod memory_monitor;
mod plugin;

pub use memory_diagnostic::*;
pub use connection_cleanup::*;
pub use message_cleanup::*;
pub use memory_monitor::*;
pub use plugin::*;

/// Re-export the main plugin
pub use plugin::NetworkMemoryPlugin;
