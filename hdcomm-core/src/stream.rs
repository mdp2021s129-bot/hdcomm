/// Stream definitions.
use serde::{Deserialize, Serialize};

/// Representation of a stream message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message {
    pub payload: Payload,
}

/// Payload of a stream message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Payload {}
