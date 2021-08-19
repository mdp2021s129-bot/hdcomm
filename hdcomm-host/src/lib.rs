mod channel;
mod codec;
pub mod error;
pub mod proxy;
pub mod router;

use futures::StreamExt;

pub async fn connect(
    path: &str,
    baud_rate: u32,
) -> Result<(router::Router, impl proxy::ControlProxy), tokio_serial::Error> {
    let framed = channel::new(path, baud_rate).await?;
    let (sink, stream) = framed.split();

    let router = router::Router::new(stream);
    let proxy = proxy::ControlProxyImpl::new(
        std::sync::Arc::new(tokio::sync::Mutex::new(sink)),
        router::RouterHandle::of(&router),
    );

    Ok((router, proxy))
}
