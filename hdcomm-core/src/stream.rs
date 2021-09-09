/// Stream definitions.
use serde::{Deserialize, Serialize};

/// Representation of a stream message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message {
    pub payload: Payload,
}

/// Payload of a stream message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Payload {
    /// Payload contains an AHRS sample.
    Ahrs(AhrsBody),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
/// Ahrs sample.
///
/// All arrays are `[x, y, z]`.
///
/// Interpretation of the values are left up to uppers layers.
pub struct AhrsBody {
    /// Acceleration.
    pub acc: [i16; 3],
    /// Angular velocity.
    pub gyro: [i16; 3],
    /// Magnetometer.
    pub mag: [i16; 3],
    /// Device timestamp.
    pub time_ms: u32,
}
