/// hdcomm configuration.
use hdcomm_core::rpc::PidParams;
use serde::{Deserialize, Serialize};

/// hdcomm server configuration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    /// gRPC server config.
    pub server: Server,
    /// serial port config.
    pub serial: Serial,
    /// Robot model configuration.
    pub model: Model,
    /// Motion control configuration.
    pub motion: Motion,
}

/// gRPC Server configuration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Server {
    /// Listening port number.
    pub port: u16,
}

/// Serial port configuration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Serial {
    /// Serial port name.
    pub name: String,
    /// Serial port baud rate.
    pub baud: u32,
}

/// Robot model configuration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Model {
    /// Encoder counts per metre of movement.
    pub counts_per_metre: f64,
    /// Distance between center of mass to the line connecting both rear
    /// wheels.
    ///
    /// In units of metres.
    pub a2: f64,
    /// Shortest distance between the line connecting both front wheels to the
    /// line connecting both rear wheels.
    ///
    /// In units of metres.
    pub l: f64,
    /// Distance between the centers of both rear wheels.
    ///
    /// In units of metres.
    pub w: f64,
    /// Available turn radii for the robot.
    pub turn_radii: Box<[TurnRadius]>,
    /// Neutral steering control signal for the robot.
    pub neutral_control: f64,
}

/// Turn radius specification.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TurnRadius {
    /// Turn radius in metres.
    pub radius: f64,
    /// Servo control signal for a left turn.
    pub control_left: f64,
    /// Servo control signal for a right turn.
    pub control_right: f64,
}

/// Motion control configuration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Motion {
    /// PID parameters for the left wheel's position control loop.
    pub pid_left: PidParams,
    /// PID parameters for the right wheel's position control loop.
    pub pid_right: PidParams,
    /// Position control loop update interval.
    ///
    /// In units of seconds.
    pub pid_update_interval: f64,
    /// Max jerk in ms^-3.
    pub max_jerk: f64,
    /// Max acceleration in ms^-2.
    pub max_accel: f64,
    /// Max velocity in ms^-2.
    pub max_velocity: f64,
    /// Time delay for steering setup (seconds).
    pub steering_setup_time: f64,
}
