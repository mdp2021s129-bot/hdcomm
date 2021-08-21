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
    PingReq(PingReqBody),
    PingRep(PingRepBody),
    EndReq(EndReqBody),
}

pub type PingReqBody = ();
pub type PingRepBody = ();
pub type EndReqBody = ();
