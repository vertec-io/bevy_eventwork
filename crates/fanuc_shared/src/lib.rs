//! Shared types for FANUC robot control example
//! 
//! This crate defines component types that are shared between the Bevy server
//! and the Leptos web client. It uses conditional compilation to include Bevy
//! traits only when building for the server.

use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use bevy::prelude::*;

// Re-export for convenience
pub use axis::*;
pub use commands::*;
pub use state::*;

/// Robot state components
pub mod state {
    use super::*;

    /// Cartesian position and orientation of the robot end effector
    #[cfg_attr(feature = "server", derive(Component, Reflect))]
    #[cfg_attr(feature = "server", reflect(Component))]
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub struct RobotPosition {
        /// X coordinate in mm
        pub x: f32,
        /// Y coordinate in mm
        pub y: f32,
        /// Z coordinate in mm
        pub z: f32,
        /// W (wrist) rotation in degrees
        pub w: f32,
        /// P (pitch) rotation in degrees
        pub p: f32,
        /// R (roll) rotation in degrees
        pub r: f32,
    }

    impl Default for RobotPosition {
        fn default() -> Self {
            Self {
                x: 0.0,
                y: 0.0,
                z: 400.0, // Start at 400mm above origin
                w: 0.0,
                p: 0.0,
                r: 0.0,
            }
        }
    }

    /// Robot operational status
    #[cfg_attr(feature = "server", derive(Component, Reflect))]
    #[cfg_attr(feature = "server", reflect(Component))]
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub struct RobotStatus {
        /// Servos are powered and ready
        pub servo_ready: bool,
        /// Teach pendant is enabled
        pub tp_enabled: bool,
        /// Robot is currently executing motion
        pub in_motion: bool,
        /// Error code (None = no error)
        pub error_code: Option<u32>,
    }

    impl Default for RobotStatus {
        fn default() -> Self {
            Self {
                servo_ready: true,
                tp_enabled: false,
                in_motion: false,
                error_code: None,
            }
        }
    }

    /// Joint angles in degrees
    #[cfg_attr(feature = "server", derive(Component, Reflect))]
    #[cfg_attr(feature = "server", reflect(Component))]
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub struct JointAngles {
        pub j1: f32,
        pub j2: f32,
        pub j3: f32,
        pub j4: f32,
        pub j5: f32,
        pub j6: f32,
    }

    impl Default for JointAngles {
        fn default() -> Self {
            Self {
                j1: 0.0,
                j2: 0.0,
                j3: 0.0,
                j4: 0.0,
                j5: 0.0,
                j6: 0.0,
            }
        }
    }

    /// Robot identification
    #[cfg_attr(feature = "server", derive(Component, Reflect))]
    #[cfg_attr(feature = "server", reflect(Component))]
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
}

/// Command types for robot control
pub mod commands {
    use super::*;

    /// Jog command for incremental movement
    #[cfg_attr(feature = "server", derive(Component, Reflect))]
    #[cfg_attr(feature = "server", reflect(Component))]
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub struct JogCommand {
        pub axis: RobotAxis,
        pub direction: JogDirection,
        /// Speed in mm/s or deg/s
        pub speed: f32,
        /// Distance in mm or degrees
        pub distance: f32,
    }
}

/// Axis and direction enums
pub mod axis {
    use super::*;

    #[cfg_attr(feature = "server", derive(Reflect))]
    #[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub enum RobotAxis {
        X,
        Y,
        Z,
        W,
        P,
        R,
    }

    #[cfg_attr(feature = "server", derive(Reflect))]
    #[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub enum JogDirection {
        Positive,
        Negative,
    }

    impl JogDirection {
        pub fn sign(&self) -> f32 {
            match self {
                JogDirection::Positive => 1.0,
                JogDirection::Negative => -1.0,
            }
        }
    }
}

