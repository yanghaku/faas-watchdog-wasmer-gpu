/// build the server for given handler and block to listen connections
macro_rules! build_and_serve {
    ($name:expr,$addr:expr,$num_thread:expr,$svc:expr) => {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads($num_thread)
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                hyper::Server::bind(&$addr)
                    .serve($svc)
                    .with_graceful_shutdown(shutdown_signal($name))
                    .await
            })
            .unwrap();
    };
}

/// watchdog server
mod watchdog;

/// metrics server
mod metrics;

use std::net::{IpAddr, SocketAddr};
use std::thread;

use anyhow::Result;
use log::{error, info};
use tokio::signal::ctrl_c;

use crate::WatchdogConfig;

const DEFAULT_IP_STR: &str = "0.0.0.0";

/// start the watchdog server and metrics server
pub(crate) fn start_server(config: WatchdogConfig) -> Result<()> {
    info!("Watchdog mode: {}", config._operational_mode);

    let default_ip: IpAddr = DEFAULT_IP_STR.parse().unwrap();

    let watchdog_addr = SocketAddr::new(default_ip.clone(), config._tcp_port);
    let metrics_addr = SocketAddr::new(default_ip, config._metrics_port);

    info!("Metrics listening on port: {}", config._metrics_port);
    // start the metrics server in another thread
    let metrics_config = config.clone();
    thread::Builder::new().spawn(move || {
        // metrics only use 1 threads
        if let Err(e) = metrics::build_and_serve("metrics", metrics_addr, 1, metrics_config) {
            error!("Metrics server error! {}", e);
            // stop process
            std::process::exit(1);
        }
    })?;

    // generate the request handler
    info!("Listening on http://{}", watchdog_addr);
    // block in current thread
    let num_thread = num_cpus::get();
    // default use the cpus number as thread num
    watchdog::build_and_serve("watchdog", watchdog_addr, num_thread, config)
}

/// wait for ctrl+c signal
async fn shutdown_signal(server_name: &'static str) {
    ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
    info!("{} server shutdown", server_name);
}
