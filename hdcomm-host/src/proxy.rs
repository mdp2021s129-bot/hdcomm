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
use hdcomm_core::rpc::{self, *};
use std::sync::Arc;

/// Macro declaring remote procedures.
macro_rules! remote_procedures {
    (
        $($name:ident, $request_body:path, $response_body:path);+
    ) => {
        #[async_trait]
        pub trait Proxy: Clone {
        $(async fn $name(&self, body: $request_body) -> Result<$response_body, RPCError>;)+
        }
    };
}

remote_procedures!(
    ping, PingReqBody, PingRepBody;
    move_cmd, MoveReqBody, MoveRepBody;
    move_status, MoveStatusReqBody, MoveStatusRepBody;
    move_cancel, MoveCancelReqBody, MoveCancelRepBody;
    pid_param_update, PidParamUpdateReqBody, PidParamUpdateRepBody;
    raw_teleop, RawTeleOpReqBody, RawTeleOpRepBody
);

/// `ProxyImpl` implements a RPC proxy.
#[derive(Clone)]
pub struct ProxyImpl {
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

/// Macro defining a remote procedure.
///
/// - `name`: name of remote method
/// - `request`: enum variant of RPC request
/// - `request_body`: type of request body
/// - `response`: enum variant of RPC response
/// - `response_body`: type of response body.
macro_rules! remote_procedure_impl {
    (
        $($name:ident, $request:path, $request_body:path, $response:path, $response_body:path);+
    ) => {
        #[async_trait]
        impl Proxy for ProxyImpl {
            $(async fn $name(&self, body: $request_body) -> Result<$response_body, RPCError> {
                let id = self.gen_id();

                let message = Message {
                    payload: message::Payload::RPC(rpc::Message {
                        id,
                        payload: $request(body),
                    }),
                };

                // Being unable to subscribe can only becaused by having too many
                // RPCs in flight.
                let receiver = self
                    .router
                    .subscribe_rpc(id)
                    .map_err(|_| RPCError::TooManyInFlight)?;

                {
                    let mut sink = self.sink.lock().await;
                    sink.send(message).await?;
                }

                // Receive errors here can only be the result of disconnection.
                let response = receiver.await.map_err(|_| RPCError::Disconnected)?;

                match response {
                    $response(resp_body) => Ok(resp_body),
                    _ => Err(RPCError::BadResponse),
                }
            })+
        }
    };
}

remote_procedure_impl!(
    ping, Payload::PingReq, PingReqBody, Payload::PingRep, PingRepBody;
    move_cmd, Payload::MoveReq, MoveReqBody, Payload::MoveRep, MoveRepBody;
    move_status, Payload::MoveStatusReq, MoveStatusReqBody, Payload::MoveStatusRep, MoveStatusRepBody;
    move_cancel, Payload::MoveCancelReq, MoveCancelReqBody, Payload::MoveCancelRep, MoveCancelRepBody;
    pid_param_update, Payload::PidParamUpdateReq, PidParamUpdateReqBody, Payload::PidParamUpdateRep, PidParamUpdateRepBody;
    raw_teleop, Payload::RawTeleOpReq, RawTeleOpReqBody, Payload::RawTeleOpRep, RawTeleOpRepBody
);
