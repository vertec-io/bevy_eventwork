//! Shared types for the control demo example.

use serde::{Deserialize, Serialize};

/// A simple robot component that can be controlled by clients.
#[cfg_attr(feature = "server", derive(bevy::prelude::Component))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Robot {
    pub name: String,
    pub x: f32,
    pub y: f32,
}

/// Robot status information.
#[cfg_attr(feature = "server", derive(bevy::prelude::Component))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RobotStatus {
    pub battery: f32,
    pub is_moving: bool,
}

/// Command to move a robot.
#[cfg_attr(feature = "server", derive(bevy::prelude::Message))]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MoveCommand {
    pub entity_id: u64,
    pub target_x: f32,
    pub target_y: f32,
}

