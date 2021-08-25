/// Definitions for messages exchanged between the host and device.
use crate::{rpc, stream};
use serde::{Deserialize, Serialize};

/// `Message` represents a single exchanged message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message {
    /// Message payload.
    pub payload: Payload,
    // Reserved for future extensions.
}

/// `Content` represents the message content.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Payload {
    /// Message is an RPC message.
    RPC(rpc::Message),
    /// Message is a stream message.
    Stream(stream::Message),
}
