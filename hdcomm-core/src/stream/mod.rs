/// Stream message payloads.
pub mod application;
use serde::{Deserialize, Serialize};

/// Content of a stream message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Content {
    pub payload: Payload,
}

/// Payload of a stream message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Payload {
    Application(application::Payload),
}
