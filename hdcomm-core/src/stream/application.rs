/// Application stream payload definitions.
use serde::{Deserialize, Serialize};

/// The payload of an application-level stream message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Payload {}
