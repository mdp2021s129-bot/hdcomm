/// Codec implementation for HdComm messages.
use crate::error::CodecError;
use bytes::{Buf, BytesMut};
use hdcomm_core::message::Message;
use postcard::{CobsAccumulator, FeedResult};
use tokio_util::codec::{Decoder, Encoder};

/// Codec is a codec that ensures all transmitted data over the serial line is
/// in the form of messages.
///
/// `N` specifies the COBS decoder's buffer size in bytes.
///
/// On transmitting, encodes all `Message`s using COBS-framed Postcard
/// serialization.
///
/// On receiving, recovers COBS frames and decodes deserializes as Postcard
/// serialized `Message`s.
///
// TODO: add better error handling?
/// If the COBS buffer is full, existing data in the buffer is dropped.
/// If a deserialization error occurs, the buffered data is dropped.
pub struct Codec<const N: usize> {
    decoder: CobsAccumulator<N>,
}

impl<const N: usize> Encoder<Message> for Codec<N> {
    type Error = CodecError;
    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // TODO: Make more efficient.
        let buf = postcard::to_stdvec_cobs(&item)?;
        dst.extend_from_slice(&buf);
        Ok(())
    }
}

/// Returns the byte distances between `p2` & `p1`, evaluated as
/// `p2` - `p1`.
///
/// Error if `p2` < `p1`.
fn offset_from(p2: *const u8, p1: *const u8) -> usize {
    p2 as usize - p1 as usize
}

impl<const N: usize> Decoder for Codec<N> {
    type Item = Message;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut trim_at: Option<usize> = None;
        let ret = match self.decoder.feed::<Message>(src) {
            FeedResult::Consumed => Ok(None),
            FeedResult::OverFull(remaining) => {
                trim_at = Some(offset_from(remaining.as_ptr(), src.as_ptr()));
                Err(Self::Error::FrameOverflow)
            }
            FeedResult::DeserError(remaining) => {
                trim_at = Some(offset_from(remaining.as_ptr(), src.as_ptr()));
                Err(Self::Error::Deserialization)
            }
            FeedResult::Success { data, remaining } => {
                trim_at = Some(offset_from(remaining.as_ptr(), src.as_ptr()));
                Ok(Some(data))
            }
        };

        if let Some(idx) = trim_at {
            src.advance(idx)
        };

        ret
    }
}

impl<const N: usize> Default for Codec<N> {
    fn default() -> Self {
        Self {
            decoder: CobsAccumulator::new(),
        }
    }
}
