/// hdcomm host-side error definitions.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RPCError {
    /// Remote disconnected.
    /// The remote could have:
    /// - Timed out
    /// - Elected to disconnect
    #[error("device disconnected")]
    Disconnected,
    #[error("too many RPCs in flight")]
    TooManyInFlight,
    #[error("codec: {0}")]
    Codec(#[from] CodecError),
}

/// Errors returned by the Codec when writing / reading `Message`s from a
/// framed channel.
///
/// All errors except I/O errors are recoverable. If an I/O error is
/// encountered, callers must be prepared to recreate the channel.
#[derive(Error, Debug)]
pub enum CodecError {
    #[error("I/O: {0}")]
    IO(#[from] std::io::Error),
    #[error("receive frame buffer overflow")]
    FrameOverflow,
    #[error("deserialization")]
    Deserialization,
    #[error("serialization: {0}")]
    Serialization(#[from] postcard::Error),
}
