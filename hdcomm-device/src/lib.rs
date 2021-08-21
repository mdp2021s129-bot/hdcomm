#![no_std]
use core::ops::{Index, IndexMut};

use hdcomm_core::message::Message;
use postcard::{
    flavors::{Cobs, SerFlavor},
    CobsAccumulator, FeedResult,
};
use serde::Serialize;

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

/// The `HVecRef` flavor is a wrapper type around a reference to a `heapless::Vec`.
pub struct HVecRef<'a, const B: usize>(&'a mut heapless::Vec<u8, B>);

impl<const B: usize> IndexMut<usize> for HVecRef<'_, B> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<const B: usize> Index<usize> for HVecRef<'_, B> {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}
/// `SerFlavor` implementation for `HVecRef`.
///
/// Does not return anything because the caller is expected to have access
/// to the target vector.
impl<const B: usize> SerFlavor for HVecRef<'_, B> {
    type Output = ();

    #[inline(always)]
    fn try_extend(&mut self, data: &[u8]) -> core::result::Result<(), ()> {
        self.0.extend_from_slice(data)
    }

    #[inline(always)]
    fn try_push(&mut self, data: u8) -> core::result::Result<(), ()> {
        self.0.push(data).map_err(|_| ())
    }

    fn release(self) -> core::result::Result<Self::Output, ()> {
        Ok(())
    }
}

/// Serializes a `Message` into an existing heapless Vec.
pub fn into_vec<T, const N: usize>(
    value: &T,
    result: &mut heapless::Vec<u8, N>,
) -> postcard::Result<()>
where
    T: Serialize + ?Sized,
{
    postcard::serialize_with_flavor::<T, _, _>(value, Cobs::try_new(HVecRef(result))?)
}
