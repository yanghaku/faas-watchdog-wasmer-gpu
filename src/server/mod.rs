mod watchdog;
mod metrics;


use std::net::{IpAddr, SocketAddr};
use std::thread;

use anyhow::Result;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use tokio::runtime;
use tokio::signal::ctrl_c;

use crate::WatchdogConfig;


pub(crate) trait Handler {
    fn handle(&mut self, _: Request<Body>) -> Result<Response<Body>, hyper::Error>;
}


const DEFAULT_IP_STR: &str = "127.0.0.1";


/// start the watchdog server and metrics server
pub(crate) fn start_server(config: WatchdogConfig) -> Result<()> {
    println!("Watchdog mode: {}", config._operational_mode);

    let default_ip: IpAddr = DEFAULT_IP_STR.parse().unwrap();

    let watchdog_addr = SocketAddr::new(default_ip.clone(), config._tcp_port);
    let metrics_addr = SocketAddr::new(default_ip, config._metrics_port);
    println!("Listening on http://{}", watchdog_addr);
    println!("Metrics listening on port: {}", config._metrics_port);


    let metrics_handler = metrics::make_handler(config.clone())?;
    // start the metrics server in another thread
    thread::Builder::new().spawn(move || {
        build_and_serve("metrics", metrics_addr, metrics_handler);
    })?;


    // generate the request handler
    let watchdog_handler = watchdog::make_handler(config)?;
    // block in current thread
    build_and_serve("watchdog", watchdog_addr, watchdog_handler);

    Ok(())
}


/// build the server for given handler and block to listen connections
fn build_and_serve<H: 'static + Handler + Send>(name: &'static str, addr: SocketAddr, mut handler: H) {
    // todo: fix the FnOnce to FnMut
    let service = move |req: _| async move {
        handler.handle(req)
    };

    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            Server::bind(&addr)
                .serve(make_service_fn(|_| async move {
                    Ok::<_, hyper::Error>(service_fn(service))
                }))
                .with_graceful_shutdown(shutdown_signal(name)).await
        }).unwrap();
}


/// wait for ctrl+c signal
async fn shutdown_signal(server_name: &'static str) {
    ctrl_c().await.expect("failed to install CTRL+C signal handler");
    println!("{} server shutdown", server_name);
}
