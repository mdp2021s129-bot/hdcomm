use hdcomm::config::Config;
use hdcomm_core::stream::Payload;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut config = config::Config::new();
    config.merge(config::File::new("hdcomm", config::FileFormat::Toml))?;
    let config: Config = config.try_into()?;
    log::info!("loaded configuration: {:?}", config);

    let (mut router, proxy) = hdcomm_host::connect(&config.serial.name, config.serial.baud).await?;

    tokio::spawn(async move { router.run().await });
    let mut stream = proxy.subscribe();

    let mut filter = hdcomm::ahrs::Filter::new(&config.ahrs);

    loop {
        let msg = stream.recv().await?;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let Payload::Ahrs(raw) = msg;
        filter.update(&raw);
        println!("{}, {}", ts, filter.euler_angles().yaw);
    }

    Ok(())
}
