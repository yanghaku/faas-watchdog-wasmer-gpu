use std::sync::Arc;

use anyhow::{anyhow, Result};
use hyper::{Body, Method, Request, Response, StatusCode};
use log::error;

use super::Handler;
use crate::{WatchdogConfig, WatchdogMode, check_healthy};
use crate::runner::{ForkingRunner, HttpRunner, Runner, SerializingForkRunner, StaticFileProcessor};

#[cfg(feature = "wasm")]
use crate::runner::WasmRunner;


struct WatchdogHandler<R> where R: Runner {
    runner: R,
}


impl<R> Handler for WatchdogHandler<R> where R: Runner {
    fn handle(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let mut response = Response::default(); // default is 200 OK

        if req.method() == &Method::OPTIONS { // for options methods, just return accept
            response.headers_mut().insert("Access-Control-Allow-Headers", "*".parse().unwrap());
            response.headers_mut().insert("Access-Control-Allow-Origin", "*".parse().unwrap());
            return Ok(response);
        }

        match req.uri().path() {
            "/_/health" => { // check healthy
                if req.method() == &Method::GET {
                    if check_healthy() {
                        *response.body_mut() = Body::from("OK");
                    } else {
                        *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
                    }
                } else {// other methods are not allowed
                    *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
                }
            }
            _ => { // for every other path and method
                if let Err(ref err) = self.runner.run(req, &mut response) {
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    *response.body_mut() = Body::from(err.to_string());
                    error!("{}", err.to_string());
                }
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

        WatchdogMode::ModeWasm => {
            #[cfg(feature = "wasm")]{
                Ok(Arc::new(WatchdogHandler { runner: WasmRunner::new(config)? }))
            }
            #[cfg(not(feature = "wasm"))]{
                Err(anyhow!("`wasm` feature doest not be enable"))
            }
        }

        _ => Err(anyhow!("watchdog mode {} is not yet implemented", config._operational_mode))
    };
}
