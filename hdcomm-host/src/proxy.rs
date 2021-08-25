use crate::channel::FramedChannel;
/// RPC proxy objects.
///
/// Drop all proxies to terminate the device -> host side of the connection.
use crate::error::RPCError;
use crate::router::RouterHandle;
use async_trait::async_trait;
use futures::stream::SplitSink;
use futures::SinkExt;
use hdcomm_core::message::{self, Message};
use hdcomm_core::rpc;
use std::sync::Arc;

/// `Proxy` exposes RPC requests.
#[async_trait]
pub trait Proxy {
    async fn ping(&self) -> Result<rpc::PingRepBody, RPCError>;
    async fn end(&self) -> Result<(), RPCError>;
}

/// `ProxyImpl` implements a RPC proxy.
#[derive(Clone)]
pub(crate) struct ProxyImpl {
    sink: Arc<tokio::sync::Mutex<SplitSink<FramedChannel, Message>>>,
    id: Arc<std::sync::Mutex<u16>>,
    router: RouterHandle,
}

impl ProxyImpl {
    pub(crate) fn new(
        sink: Arc<tokio::sync::Mutex<SplitSink<FramedChannel, Message>>>,
        router: RouterHandle,
    ) -> Self {
        Self {
            sink,
            id: Arc::new(std::sync::Mutex::new(0)),
            router,
        }
    }

    fn gen_id(&self) -> u16 {
        let mut id = self.id.lock().unwrap();
        *id = id.wrapping_add(1);
        *id
    }
}

#[async_trait]
impl Proxy for ProxyImpl {
    async fn end(&self) -> Result<(), RPCError> {
        let id = self.gen_id();

        let message = Message {
            payload: message::Payload::RPC(rpc::Message {
                id,
                payload: rpc::Payload::EndReq(()),
            }),
        };

        self.sink.lock().await.send(message).await?;

        Ok(())
    }

    async fn ping(&self) -> Result<rpc::PingRepBody, RPCError> {
        let id = self.gen_id();

        let message = Message {
            payload: message::Payload::RPC(rpc::Message {
                id,
                payload: rpc::Payload::PingReq(()),
            }),
        };

        let receiver = self
            .router
            .subscribe_rpc(id)
            .map_err(|_| RPCError::TooManyInFlight)?;

        self.sink.lock().await.send(message).await?;

        // A receive error here can only be the result of disconnection.
        receiver.await.map_err(|_| RPCError::Disconnected)?;

        Ok(())
    }
}
