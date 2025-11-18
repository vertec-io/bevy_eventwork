//! Shared types for FANUC robot control using real FANUC_RMI_API
//! 
//! This crate provides wrapper types around FANUC_RMI_API DTO types that can be
//! used as Bevy components and synchronized via eventwork_sync.

use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[cfg(feature = "server")]
use bevy::prelude::*;

// Re-export FANUC types for convenience
pub use fanuc_rmi::dto;

/// Wrapper around fanuc_rmi::dto::Position for use as a Bevy component
#[cfg_attr(feature = "server", derive(Component))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RobotPosition(pub dto::Position);

impl Deref for RobotPosition {
    type Target = dto::Position;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RobotPosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for RobotPosition {
    fn default() -> Self {
        Self(dto::Position {
            x: 0.0,
            y: 0.0,
            z: 400.0, // Start at 400mm above origin
            w: 0.0,
            p: 0.0,
            r: 0.0,
            ext1: 0.0,
            ext2: 0.0,
            ext3: 0.0,
        })
    }
}

/// Wrapper around fanuc_rmi::dto::JointAngles for use as a Bevy component
#[cfg_attr(feature = "server", derive(Component))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct JointAngles(pub dto::JointAngles);

impl Deref for JointAngles {
    type Target = dto::JointAngles;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for JointAngles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for JointAngles {
    fn default() -> Self {
        Self(dto::JointAngles {
            j1: 0.0,
            j2: 0.0,
            j3: 0.0,
            j4: 0.0,
            j5: 0.0,
            j6: 0.0,
            j7: 0.0,
            j8: 0.0,
            j9: 0.0,
        })
    }
}

/// Robot operational status
#[cfg_attr(feature = "server", derive(Component))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RobotStatus {
    /// Servos are powered and ready
    pub servo_ready: bool,
    /// Teach pendant is enabled
    pub tp_enabled: bool,
    /// Robot is currently executing motion
    pub in_motion: bool,
    /// Error message (None = no error)
    pub error_message: Option<String>,
}

impl Default for RobotStatus {
    fn default() -> Self {
        Self {
            servo_ready: true,
            tp_enabled: false,
            in_motion: false,
            error_message: None,
        }
    }
}

/// Robot identification
#[cfg_attr(feature = "server", derive(Component))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RobotInfo {
    pub name: String,
    pub model: String,
}

impl Default for RobotInfo {
    fn default() -> Self {
        Self {
            name: "FANUC Robot".to_string(),
            model: "LR Mate 200iD".to_string(),
        }
    }
}

/// Motion command to be sent to the robot
#[cfg_attr(feature = "server", derive(Component))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MotionCommand {
    pub instruction: dto::Instruction,
}

impl Deref for MotionCommand {
    type Target = dto::Instruction;
    fn deref(&self) -> &Self::Target {
        &self.instruction
    }
}

impl DerefMut for MotionCommand {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.instruction
    }
}

