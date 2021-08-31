use hdcomm_core::rpc::{PidParamUpdateReqBody, PidParams};
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

    let pid_params = PidParams {
        kp: 0.01,
        ki: 0.00004,
        kd: 0.01,
        p_limit: 1.,
        i_limit: 1.,
        d_limit: 0.,
        output_limit: 1.,
    };
    let body = PidParamUpdateReqBody {
        params: [pid_params.clone(), pid_params],
        update_interval_ms: 10,
    };

    println!("PID update: {:?}", proxy.pid_param_update(body).await);

    Ok(())
}
