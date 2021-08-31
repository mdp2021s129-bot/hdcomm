use hdcomm_host::proxy::Proxy;

/// Attempts to synchronize with the target.
async fn synchronize(proxy: &impl Proxy) {
    loop {
        match tokio::time::timeout(std::time::Duration::from_secs(1), proxy.ping(())).await {
            Err(_) => eprintln!("synchronize: timed out"),
            Ok(res) => {
                if let Err(_) = res {
                    eprintln!("synchronize: RPC error")
                }
                return;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut router, proxy) = hdcomm_host::connect("/dev/ttyACM0", 921600).await.unwrap();
    tokio::spawn(async move { router.run().await });

    synchronize(&proxy).await;

    println!("Move status: {:?}", proxy.move_status(()).await);

    Ok(())
}
