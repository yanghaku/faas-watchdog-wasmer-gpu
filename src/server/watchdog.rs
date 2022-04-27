use std::sync::Arc;

use anyhow::Result;
use hyper::{Body, Method, Request, Response, StatusCode};

use crate::config::WatchdogMode;
use crate::server::Handler;
use crate::WatchdogConfig;
use crate::health::check_healthy;
use crate::runner::{ForkingRunner, HttpRunner, Runner, SerializingForkRunner, StaticFileProcessor};

#[cfg(feature = "wasm")]
use crate::runner::WasmRunner;


struct WatchdogHandler<R> where R: Runner {
    runner: R,
}


impl<R> Handler for WatchdogHandler<R> where R: Runner {
    fn handle(&self, mut req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let mut response = Response::default(); // default is 200 OK

        match req.method() {
            &Method::GET => {
                match req.uri().path() {
                    "/" => {
                        if let Err(ref err) = self.runner.run(&mut req, &mut response) {
                            eprintln!("{}", err.to_string());
                        }
                    }
                    "/_/health" => { // check healthy
                        if check_healthy() {
                            *response.body_mut() = Body::from("OK");
                        } else {
                            *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
                        }
                    }
                    _ => { // 404 not found
                        *response.status_mut() = StatusCode::NOT_FOUND;
                    }
                }
            }
            &Method::OPTIONS => {// for options methods, just return accept
                response.headers_mut().insert("Access-Control-Allow-Headers", "*".parse().unwrap());
                response.headers_mut().insert("Access-Control-Allow-Origin", "*".parse().unwrap());
            }
            _ => { // other methods are not allowed
                *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
            }
        }

        Ok(response)
    }
}


pub(super) fn make_handler(config: WatchdogConfig) -> Result<Arc<dyn Handler + Send + Sync>> {
    return match config._operational_mode {
        WatchdogMode::ModeStreaming =>
            Ok(Arc::new(WatchdogHandler { runner: ForkingRunner::new(config)? })),

        WatchdogMode::ModeHTTP =>
            Ok(Arc::new(WatchdogHandler { runner: HttpRunner::new(config)? })),

        WatchdogMode::ModeStatic =>
            Ok(Arc::new(WatchdogHandler { runner: StaticFileProcessor::new(config)? })),

        WatchdogMode::ModeSerializing =>
            Ok(Arc::new(WatchdogHandler { runner: SerializingForkRunner::new(config)? })),

        #[cfg(feature = "wasm")]
        WatchdogMode::ModeWasm =>
            Ok(Arc::new(WatchdogHandler { runner: WasmRunner::new(config)? })),

        _ => Err(anyhow::Error::msg(
            format!("watchdog mode {} is not yet implemented", config._operational_mode)))
    };
}
