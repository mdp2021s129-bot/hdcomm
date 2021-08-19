/// Application message payload definitions.
use serde::{Deserialize, Serialize};

/// The payload of an application-level RPC message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Payload {
    PwmReq(PwmReqBody),
    PwmRep(PwmRepBody),
}

/// PWM control request body.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PwmReqBody {
    /// Red component duty cycle.
    pub r: u8,
    /// Green component duty cycle.
    pub g: u8,
    /// Blue component duty cycle.
    pub b: u8,
}

/// PWM control reply body.
pub type PwmRepBody = ();
