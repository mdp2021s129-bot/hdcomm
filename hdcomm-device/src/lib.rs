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
