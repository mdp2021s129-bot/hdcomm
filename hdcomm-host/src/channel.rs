/// Underlying transport channel for RPCs.
use crate::codec::Codec;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::Framed;

/// Type of the framed transport channel.
pub(crate) type FramedChannel = Framed<SerialStream, Codec<256>>;

/// Creates a new framed transport channel using a given serial port and the
/// provided baud rate.
pub(crate) async fn new(path: &str, baud_rate: u32) -> Result<FramedChannel, tokio_serial::Error> {
    let stream = tokio_serial::new(path, baud_rate)
        .parity(tokio_serial::Parity::None)
        .data_bits(tokio_serial::DataBits::Eight)
        .stop_bits(tokio_serial::StopBits::One)
        .flow_control(tokio_serial::FlowControl::None)
        .open_native_async()?;

    Ok(Framed::new(stream, Codec::default()))
}
