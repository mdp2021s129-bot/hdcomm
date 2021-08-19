/// Control message payload definitions.
use serde::{Deserialize, Serialize};

/// The payload of a control RPC message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Payload {
    PingReq(PingReqBody),
    PingRep(PingRepBody),
    EndReq(EndReqBody),
}

/// Ping request body.
pub type PingReqBody = ();
/// Ping reply body.
pub type PingRepBody = ();
/// End request body.
pub type EndReqBody = ();
