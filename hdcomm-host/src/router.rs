use crate::channel::FramedChannel;
use futures::stream::{SplitStream, StreamExt};
/// Router that routes responses from a framed channel to receivers.
use hdcomm_core::{
    message::{self, Message},
    rpc, stream,
};
use std::collections::{hash_map::Entry, HashMap};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, oneshot};

/// Listeners that are waiting for messages from the router.
struct Listeners {
    /// Destination for control messages received from the device.
    control_rpc: HashMap<u16, oneshot::Sender<rpc::control::Payload>>,
    /// Destination for application-level messsages received from the device.
    application_rpc: HashMap<u16, oneshot::Sender<rpc::application::Payload>>,
    /// Destination for application-level streaming messages received from the
    /// device.
    application_stream: broadcast::Sender<stream::application::Payload>,
}

impl Default for Listeners {
    fn default() -> Self {
        let (application_stream, _) = broadcast::channel(1024);
        Self {
            control_rpc: HashMap::new(),
            application_rpc: HashMap::new(),
            application_stream,
        }
    }
}

/// The router retrieves messages from a channel, then dispatches it to the
/// appropriate listener based on its type.
///
/// Drop the `Router` to terminate the Device -> host half of the connection.
pub struct Router {
    /// Incoming message source.
    incoming: SplitStream<FramedChannel>,

    /// Message listeners.
    listeners: Arc<Mutex<Listeners>>,
}

impl Router {
    /// Create a new router that routes messages from the given source.
    pub(crate) fn new(incoming: SplitStream<FramedChannel>) -> Self {
        Self {
            incoming,
            listeners: Arc::new(Mutex::new(Listeners::default())),
        }
    }

    /// Runs the router.
    /// Will never exit unless cancelled.
    pub async fn run(&mut self) {
        loop {
            let opt = self.incoming.next().await;
            let res = match opt {
                None => continue,
                Some(res) => res,
            };
            let message = match res {
                Err(_) => continue,
                Ok(message) => message,
            };

            match message {
                Message {
                    content: message::Content::RPC(rpc::Content { id, payload }),
                } => match payload {
                    rpc::Payload::Control(control_payload) => {
                        if let Some(listener) =
                            self.listeners.lock().unwrap().control_rpc.remove(&id)
                        {
                            listener.send(control_payload).ok();
                        }
                    }
                    rpc::Payload::Application(app_payload) => {
                        if let Some(listener) =
                            self.listeners.lock().unwrap().application_rpc.remove(&id)
                        {
                            listener.send(app_payload).ok();
                        }
                    }
                },
                Message {
                    content:
                        message::Content::Stream(stream::Content {
                            payload: stream::Payload::Application(app_payload),
                        }),
                } => {
                    self.listeners
                        .lock()
                        .unwrap()
                        .application_stream
                        .send(app_payload)
                        .ok();
                }
            }
        }
    }
}

/// A shared RPC router.
#[derive(Clone)]
pub(crate) struct RouterHandle {
    /// Group of listeners that this handle is bound to.
    listeners: Arc<Mutex<Listeners>>,
}

impl RouterHandle {
    /// Creates a new handle referencing a created router.
    pub(crate) fn of(router: &Router) -> Self {
        Self {
            listeners: router.listeners.clone(),
        }
    }

    /// Subscribe to a control RPC message with the given ID.
    pub(crate) fn subscribe_control_rpc(
        &self,
        id: u16,
    ) -> Result<oneshot::Receiver<rpc::control::Payload>, ()> {
        match self.listeners.lock().unwrap().control_rpc.entry(id) {
            Entry::Occupied(mut oe) => {
                if oe.get().is_closed() {
                    let (tx, rx) = oneshot::channel();
                    oe.insert(tx);
                    Ok(rx)
                } else {
                    Err(())
                }
            }
            Entry::Vacant(ve) => {
                let (tx, rx) = oneshot::channel();
                ve.insert(tx);
                Ok(rx)
            }
        }
    }

    /// Subscribe to a application RPC message with the given ID.
    pub(crate) fn subscribe_application_rpc(
        &self,
        id: u16,
    ) -> Result<oneshot::Receiver<rpc::application::Payload>, ()> {
        match self.listeners.lock().unwrap().application_rpc.entry(id) {
            Entry::Occupied(mut oe) => {
                if oe.get().is_closed() {
                    let (tx, rx) = oneshot::channel();
                    oe.insert(tx);
                    Ok(rx)
                } else {
                    Err(())
                }
            }
            Entry::Vacant(ve) => {
                let (tx, rx) = oneshot::channel();
                ve.insert(tx);
                Ok(rx)
            }
        }
    }

    /// Subscribe to application stream messagess.
    pub(crate) fn subscribe_application_stream(
        &self,
    ) -> broadcast::Receiver<stream::application::Payload> {
        self.listeners
            .lock()
            .unwrap()
            .application_stream
            .subscribe()
    }
}
