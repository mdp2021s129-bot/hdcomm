use hdcomm_core::rpc::MoveReqBody;
use hdcomm_host::proxy::Proxy;
use s_curve::{SCurveConstraints, SCurveInput, SCurveParameters, SCurveStartConditions};

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
    let constraints = SCurveConstraints {
        max_acceleration: 800.0,
        max_velocity: 800.0,
        max_jerk: 1600.0,
    };
    let start_conditions = SCurveStartConditions {
        q0: 0.,
        q1: 10000.,
        v0: 0.,
        v1: 0.,
    };
    let input = SCurveInput {
        constraints,
        start_conditions,
    };
    let time_intervals = input.calc_intervals();
    let parameters = SCurveParameters::new(&time_intervals, &input);
    let req_body = MoveReqBody {
        params: parameters,
        ref_left: false,
        ratio: 1.0,
        steering: 0.0,
        steering_setup_ms: 1000,
        reverse: true,
    };
    let time_required = req_body.time_required();

    println!("Move cmd: {:?}", proxy.move_cmd(req_body).await);
    println!("Expected completion in {} s", time_required);

    Ok(())
}
