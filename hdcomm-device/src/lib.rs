#![no_std]
use hdcomm_core::message::Message;
use postcard::{CobsAccumulator, FeedResult};

/// An accumulator that consumes message data and returns deserialized
/// `Message`s.
pub struct Accumulator(CobsAccumulator<{ hdcomm_core::MAX_MESSAGE_LENGTH }>);

impl Accumulator {
    /// Create a new accumulator.
    pub const fn new() -> Self {
        Self(CobsAccumulator::new())
    }

    /// Reset the accumulator.
    pub fn reset(&mut self) {
        self.0 = CobsAccumulator::new()
    }

    /// Feed the accumulator.
    /// See the documentation for `postcard::CobsAccumulator` for more
    /// information.
    pub fn feed<'a>(&mut self, buf: &'a [u8]) -> FeedResult<'a, Message> {
        self.0.feed(buf)
    }
}

/// Maximum number of bytes required for a serialized `Message` framed using
/// COBS.
///
/// Calculated as ceil(MAX_MESSAGE_LENGTH / 254).
pub const ENCODED_BUFFER_SIZE: usize = hdcomm_core::MAX_MESSAGE_LENGTH
    + (hdcomm_core::MAX_MESSAGE_LENGTH / 254)
    + if (hdcomm_core::MAX_MESSAGE_LENGTH % 254) > 0 {
        1
    } else {
        0
    };

/// Serializes a `Message` and writes its COBS-framed version to a buffer.
///
/// This buffer should be at least `ENCODED_BUFFER_SIZE` in length to avoid
/// errors due to insufficient outptu buffer space.
pub fn frame<'a>(message: &Message, buf: &'a mut [u8]) -> Result<&'a mut [u8], postcard::Error> {
    postcard::to_slice_cobs(message, buf)
}
