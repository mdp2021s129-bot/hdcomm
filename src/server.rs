use crate::config::Config;
use crate::model::{Error as ModelError, Model};
use crate::stream::Processor;
use hdcomm_core::rpc::{self, PidParamUpdateReqBody};
use hdcomm_host::proxy::{Proxy, ProxyImpl};
use hdcomm_server::hd_comm_server::HdComm;
use hdcomm_server::{
    FrontDistanceResponse, HeadingResponse, MoveRequest, MoveResponse, PingResponse, RadiiResponse,
};
use prost_types::Duration as GrpcDuration;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::task::JoinHandle;
use tokio_serial::Error as SerialError;
use tonic::{Request, Response, Status};

pub mod hdcomm_server {
    tonic::include_proto!("hdcomm");
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("serial initialization")]
    Serial(#[from] SerialError),
    #[error("initial parameter upload")]
    InitialParamUpload,
}

/// HdComm gRPC server implementation.
pub struct ServerImpl {
    /// Robot model.
    model: Model,
    /// Server configuration.
    config: Config,
    /// Router join handle.
    router_handle: JoinHandle<()>,
    /// Host -> device RPC proxy.
    proxy: ProxyImpl,
    /// Stream processor.
    sp: Arc<Processor>,
    /// Stream processor join handle.
    sp_handle: JoinHandle<()>,
}

impl ServerImpl {
    pub async fn new(config: &Config) -> Result<Self, Error> {
        let (mut router, proxy) =
            hdcomm_host::connect(&config.serial.name, config.serial.baud).await?;

        let router_handle = tokio::spawn(async move { router.run().await });

        let config = config.clone();

        let model = Model {
            model: config.model.clone(),
            motion: config.motion.clone(),
        };

        let sp = Arc::new(Processor::new(proxy.subscribe(), &config));
        let sp_handle = {
            let sp = sp.clone();
            tokio::spawn(async move { sp.run().await })
        };

        proxy
            .pid_param_update(PidParamUpdateReqBody {
                params: [
                    config.motion.pid_left.clone(),
                    config.motion.pid_right.clone(),
                ],
                update_interval_ms: (config.motion.pid_update_interval * 1e3) as u16,
            })
            .await
            .map_err(|_| Error::InitialParamUpload)?;

        log::info!("sent PID parameters");

        Ok(Self {
            model,
            config,
            router_handle,
            proxy,
            sp,
            sp_handle,
        })
    }
}

impl Drop for ServerImpl {
    /// A custom Drop implementation is provided that destroys all background
    /// tasks associated with the server.
    fn drop(&mut self) {
        self.router_handle.abort();
        self.sp_handle.abort();
    }
}

#[tonic::async_trait]
impl HdComm for ServerImpl {
    async fn r#move(
        &self,
        request: Request<MoveRequest>,
    ) -> Result<Response<MoveResponse>, Status> {
        log::info!("move() request: {:?}", request);

        match self
            .model
            .generate_move(request.get_ref().radius_indexed, request.get_ref().distance)
        {
            Ok(mrb) => {
                let time_required: GrpcDuration =
                    Duration::from_secs_f32(mrb.time_required()).into();

                match self.proxy.move_cmd(mrb).await {
                    Ok(rpc::MoveRepBody::Accepted) => Ok(Response::new(MoveResponse {
                        time_required: Some(time_required),
                    })),
                    Ok(rpc::MoveRepBody::Busy) => Err(Status::unavailable("move in progress")),
                    Err(e) => {
                        log::warn!("hdcomm RPC error: {}", e);
                        Err(Status::internal(e.to_string()))
                    }
                }
            }
            Err(ModelError::RadiusNotSupported) => {
                Err(Status::invalid_argument("radius not supported"))
            }
            _ => unreachable!(),
        }
    }

    async fn move_cancel(&self, _: Request<()>) -> Result<Response<()>, tonic::Status> {
        log::info!("move_cancel() request");

        if let Err(e) = self.proxy.move_cancel(()).await {
            log::warn!("hdcomm RPC error: {}", e);
            Err(Status::internal(e.to_string()))
        } else {
            Ok(Response::new(()))
        }
    }

    async fn ping(&self, _: Request<()>) -> Result<Response<PingResponse>, Status> {
        log::info!("ping() request");

        match self.proxy.ping(()).await {
            Ok(rb) => Ok(Response::new(PingResponse {
                device_time: rb.time_ms as f64 / 1e3,
            })),
            Err(e) => {
                log::warn!("hdcomm RPC error: {}", e);
                Err(Status::internal(e.to_string()))
            }
        }
    }

    async fn get_radii(&self, _: Request<()>) -> Result<Response<RadiiResponse>, Status> {
        log::info!("get_radii() request");

        let mut radii = vec![f64::INFINITY];
        radii.extend(self.model.model.turn_radii.iter().map(|r| r.radius));

        Ok(Response::new(RadiiResponse { radii }))
    }

    async fn get_heading(
        &self,
        _: tonic::Request<()>,
    ) -> Result<Response<HeadingResponse>, Status> {
        let reading = self.sp.orientation();
        let timestamp = match reading.timestamp {
            Some(ts) => ts,
            None => f64::NAN,
        };

        Ok(Response::new(HeadingResponse {
            device_time: timestamp,
            heading: reading.yaw,
        }))
    }

    async fn get_front_distance(
        &self,
        _: tonic::Request<()>,
    ) -> Result<Response<FrontDistanceResponse>, Status> {
        match self.proxy.get_front_distance(()).await {
            Ok(rb) => Ok(Response::new(FrontDistanceResponse {
                device_time_end: rb.end_time_ms as f64 / 1e3,
                device_time_start: rb.start_time_ms as f64 / 1e3,
                distance: match rb.distance {
                    Some(d) => d as f64,
                    None => f64::NAN,
                },
            })),
            Err(e) => {
                log::warn!("hdcomm RPC error: {}", e);
                Err(Status::internal(e.to_string()))
            }
        }
    }
}
