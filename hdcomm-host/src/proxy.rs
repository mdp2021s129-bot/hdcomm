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
use hdcomm_core::rpc::{self, control};
use std::sync::Arc;

/// `ControlProxy` exposes control RPC requests.
#[async_trait]
pub trait ControlProxy {
    async fn ping(&self) -> Result<control::PingRepBody, RPCError>;
    async fn end(&self) -> Result<(), RPCError>;
}

/// `ControlProxyImpl` implements a control RPC proxy.
#[derive(Clone)]
pub(crate) struct ControlProxyImpl {
    sink: Arc<tokio::sync::Mutex<SplitSink<FramedChannel, Message>>>,
    id: Arc<std::sync::Mutex<u16>>,
    router: RouterHandle,
}

impl ControlProxyImpl {
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
        *id += 1;
        *id
    }
}

#[async_trait]
impl ControlProxy for ControlProxyImpl {
    async fn end(&self) -> Result<(), RPCError> {
        let id = self.gen_id();

        let message = Message {
            content: message::Content::RPC(rpc::Content {
                id,
                payload: rpc::Payload::Control(control::Payload::EndReq(())),
            }),
        };

        self.sink.lock().await.send(message).await?;

        Ok(())
    }

    async fn ping(&self) -> Result<control::PingRepBody, RPCError> {
        let id = self.gen_id();

        let message = Message {
            content: message::Content::RPC(rpc::Content {
                id,
                payload: rpc::Payload::Control(control::Payload::PingReq(())),
            }),
        };

        let receiver = self
            .router
            .subscribe_control_rpc(id)
            .map_err(|_| RPCError::TooManyInFlight)?;

        self.sink.lock().await.send(message).await?;

        // A receive error here can only be the result of disconnection.
        receiver.await.map_err(|_| RPCError::Disconnected)?;

        Ok(())
    }
}
