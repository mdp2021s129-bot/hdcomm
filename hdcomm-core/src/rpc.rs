use s_curve::SCurveParameters;
/// RPC definitions.
use serde::{Deserialize, Serialize};

/// Representation of a RPC message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message {
    /// Message identifier.
    /// Must be the same as the originating message for replies.
    pub id: u16,
    pub payload: Payload,
}

/// The payload of a control RPC message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Payload {
    /// Ping request.
    PingReq(PingReqBody),
    PingRep(PingRepBody),

    /// Move request.
    MoveReq(MoveReqBody),
    MoveRep(MoveRepBody),

    /// Move status request.
    MoveStatusReq(MoveStatusReqBody),
    MoveStatusRep(MoveStatusRepBody),

    /// Move cancel request.
    MoveCancelReq(MoveCancelReqBody),
    MoveCancelRep(MoveCancelRepBody),

    /// PID parameters update request.
    PidParamUpdateReq(PidParamUpdateReqBody),
    PidParamUpdateRep(PidParamUpdateRepBody),

    /// Raw teleop request.
    ///
    /// Allows for direct control over steering servo and drive wheels.
    RawTeleOpReq(RawTeleOpReqBody),
    RawTeleOpRep(RawTeleOpRepBody),
}

pub type PingReqBody = ();

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PingRepBody {
    /// Time at the remote in milliseconds.
    ///
    /// All remote timestamps are relative to the start of the remote.
    pub time_ms: u32,
}

/// Body of a move request.
///
/// All units are in encoder counts.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MoveReqBody {
    /// Calculated S-Curve parameters.
    pub params: SCurveParameters,
    /// Ratio of other wheel's position relative to reference wheel.
    pub ratio: f32,
    /// True if the left wheel is the reference wheel.
    ///
    /// The right wheel is used as the reference otherwise.
    pub ref_left: bool,
    /// Steering value in `[-1, 1]`.
    pub steering: f32,
    /// Duration to wait for steering to stabilize before starting move.
    ///
    /// In units of milliseconds.
    pub steering_setup_ms: u16,
    /// Whether this move should be done in reverse.
    pub reverse: bool,
}

impl MoveReqBody {
    /// Total time required for move to complete.
    pub fn time_required(&self) -> f32 {
        self.params.time_intervals.total_duration() + (self.steering_setup_ms as f32 / 1e3)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum MoveRepBody {
    /// Controller is busy with another move.
    Busy,
    /// Move command accepted.
    Accepted,
}

pub type MoveStatusReqBody = ();

/// Body of a move status response.
///
/// All units are in encoder counts.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum MoveStatusRepBody {
    /// The robot is executing a move.
    Executing {
        /// Elapsed move time, in seconds.
        elapsed: f32,
        /// Remaining time required for move to complete (in seconds).
        remaining: f32,
    },
    /// The robot is not executing a move command, but its motors may still
    /// be running if commanded through teleop.
    NoCommand,
}

pub type MoveCancelReqBody = ();
pub type MoveCancelRepBody = ();

/// Pid parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct PidParams {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
    pub p_limit: f32,
    pub i_limit: f32,
    pub d_limit: f32,
    pub output_limit: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PidParamUpdateReqBody {
    /// Position tracking loop parameters.
    ///
    /// `[0]` is the left motor, & `[1]` is the right motor.
    pub params: [PidParams; 2],
    /// Interval between control loop updates.
    ///
    /// In units of milliseconds.
    pub update_interval_ms: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PidParamUpdateRepBody {
    /// Pid parameters updated.
    Updated,
    /// Controller is busy with another move.
    Busy,
}

/// Requests raw control over the robot's actuators.
///
/// Fields that are `None` means that the actuator corresponding to that
/// field will be left in its original state.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RawTeleOpReqBody {
    /// Sets the steering to a given position.
    ///
    /// Range: `[-1, 1]`, where `-1` is full left deflection and `1` is full
    /// right. `0` for neutral.
    pub steering: Option<f32>,

    /// Sets the left wheel's PWM duty cycle.
    ///
    /// Range `[-1, 1]`, where `-1` is 100% duty cycle, driving in reverse,
    /// and `1` is 100% duty cycle, driving forwards.
    pub wheel_l: Option<f32>,
    /// Sets the right wheel's PWM duty cycle.
    ///
    /// See `wheel_l` for information about the range of values this can take.
    pub wheel_r: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum RawTeleOpRepBody {
    /// Actuator settings applied.
    Applied,
    /// Controller is busy with a move and cannot appply teleop settings.
    Busy,
}
