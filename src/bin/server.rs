/// hdcomm server
use hdcomm::config::Config;
use hdcomm::server::{hdcomm_server::hd_comm_server::HdCommServer, ServerImpl};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut config = config::Config::new();
    config.merge(config::File::new("hdcomm", config::FileFormat::Toml))?;
    let config: Config = config.try_into()?;
    log::info!("loaded configuration: {:?}", config);

    let server = ServerImpl::new(&config).await?;
    Server::builder()
        .add_service(HdCommServer::new(server))
        .serve(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            config.server.port,
        )))
        .await?;

    Ok(())
}
