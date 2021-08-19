/// RPC message payloads.
pub mod application;
pub mod control;

use serde::{Deserialize, Serialize};

/// Content of a RPC message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Content {
    /// Message identifier.
    /// Must be the same as the originating message for replies.
    pub id: u16,
    pub payload: Payload,
}

/// Payload of a RPC message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Payload {
    Control(control::Payload),
    Application(application::Payload),
}
