#![no_std]

pub mod message;
pub mod rpc;
pub mod stream;

/// The maximum length of a message in terms of bytes.
// (FIXME: no elegant way to check yet :()
pub const MAX_MESSAGE_LENGTH: usize = 256;
