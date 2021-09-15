/// Processing for stream messages received from the device.
use crate::ahrs::{Angles, Filter};
use crate::config::Config;
use hdcomm_core::stream::Payload;
use std::sync::RwLock;
use tokio::sync::{broadcast::Receiver, Mutex};

/// Stream message processor.
///
/// Receives stream messages from the device and provides functionality
/// to retrieve the streamed data.
pub struct Processor {
    /// Stream message source.
    src: Mutex<Receiver<Payload>>,
    /// AHRS filter.
    filter: RwLock<Filter>,
}

impl Processor {
    /// Create a new stream message processor.
    pub fn new(src: Receiver<Payload>, config: &Config) -> Self {
        Self {
            src: Mutex::new(src),
            filter: RwLock::new(Filter::new(&config.ahrs)),
        }
    }

    /// Run the stream processor.
    ///
    /// Enters a processing loop. Only one `run()` entry should be active
    /// at any one time.
    ///
    /// Re-entrant calls result in panics.
    pub async fn run(&self) {
        let mut rx = self.src.try_lock().unwrap();

        loop {
            match rx.recv().await {
                Ok(msg) => match msg {
                    Payload::Ahrs(raw) => self.filter.write().unwrap().update(&raw),
                },
                Err(e) => {
                    log::warn!("receive: {}", e);
                }
            }
        }
    }

    /// Retrieve the latest orientation reading.
    pub fn orientation(&self) -> Angles {
        self.filter.read().unwrap().euler_angles()
    }
}
