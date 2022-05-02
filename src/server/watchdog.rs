use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use anyhow::{anyhow, Result};
use hyper::body::to_bytes;
use hyper::http::HeaderValue;
use hyper::service::Service;
use hyper::{Body, Method, Request, Response, StatusCode};
use lazy_static::lazy_static;
use log::error;

use crate::*;
use crate::runner::{ForkingRunner, HttpRunner, Runner, SerializingForkRunner, StaticFileProcessor};
use super::shutdown_signal;

#[cfg(feature = "wasm")]
use crate::runner::WasmRunner;


pub(super) struct WatchdogMakeSvc<R>
    where R: Runner + Clone + Send + 'static,
{
    pub(super) _runner: R,
}


impl<R, T> Service<T> for WatchdogMakeSvc<R>
    where R: Runner + Clone + Send + 'static,
{
    type Response = WatchdogService<R>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let runner = self._runner.clone();
        let fut = async move { Ok(WatchdogService { _runner: runner }) };
        Box::pin(fut)
    }
}


pub(super) struct WatchdogService<R>
    where R: Runner,
{
    _runner: R,
}


impl<R> Service<Request<Body>> for WatchdogService<R>
    where R: Runner + Clone + Send + 'static,
{
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::pin(handle(self._runner.clone(), req))
    }
}


/// handle the request
async fn handle<R: Runner>(runner: R, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut response = Response::default(); // default is 200 OK

    if req.method() == &Method::OPTIONS {
        // for options methods, just return accept
        response
            .headers_mut()
            .insert("Access-Control-Allow-Headers", CONTENT_ALLOW_ALL.clone());
        response
            .headers_mut()
            .insert("Access-Control-Allow-Origin", CONTENT_ALLOW_ALL.clone());
        return Ok(response);
    }

    match req.uri().path() {
        "/_/health" => {
            // check healthy
            if req.method() == &Method::GET {
                if check_healthy() {
                    *response.body_mut() = Body::from("OK");
                } else {
                    *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
                }
            } else {
                // other methods are not allowed
                *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
            }
        }
        "/scale-reader" => {
            let (replicas, available_replicas, invocation_count) = runner.get_scale();
            let status = ReplicaFuncStatus::new(
                replicas as u64,
                available_replicas as u64,
                invocation_count as u64,
            );

            response
                .headers_mut()
                .insert("Content-Type", JSON_CONTENT_TYPE.clone());
            *response.body_mut() = Body::from(status.into_json());
        }
        "/scale-updater" => match ScaleServiceRequest::from_json(get_body_string(req).await) {
            Ok(r) => {
                if let Err(e) = runner.set_scale(r._replicas as usize) {
                    *response.body_mut() = Body::from(e.to_string());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                }
            }
            Err(e) => {
                *response.body_mut() = Body::from(format!(
                    "Cannot parse request. Please pass valid JSON. Error={}",
                    e.to_string()
                ));
                *response.status_mut() = StatusCode::BAD_REQUEST;
            }
        },
        _ => {
            // for every other path and method
            if let Err(ref err) = runner.run(req, &mut response) {
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                *response.body_mut() = Body::from(err.to_string());
                error!("{}", err.to_string());
            }
        }
    }

    Ok(response)
}


lazy_static! {
    static ref CONTENT_ALLOW_ALL: HeaderValue = "*".parse().unwrap();
    static ref JSON_CONTENT_TYPE: HeaderValue = "application/json; charset=utf-8".parse().unwrap();
}


/// helper function, buffer the hole request body to string
async fn get_body_string(req: Request<Body>) -> Result<String> {
    let bytes = to_bytes(req.into_body()).await?;
    Ok(String::from(std::str::from_utf8(bytes.as_ref())?))
}


/// build watchdog server and serve
pub(super) fn build_and_serve(name: &'static str, addr: SocketAddr,
                              num_threads: usize, config: WatchdogConfig) -> Result<()> {
    match config._operational_mode {
        WatchdogMode::ModeStreaming => {
            let runner = ForkingRunner::new(config)?;
            build_and_serve!(name, addr, num_threads, WatchdogMakeSvc { _runner: runner });
        }

        WatchdogMode::ModeHTTP => {
            let runner = HttpRunner::new(config)?;
            build_and_serve!(name, addr, num_threads, WatchdogMakeSvc { _runner: runner });
        }

        WatchdogMode::ModeStatic => {
            let runner = StaticFileProcessor::new(config)?;
            build_and_serve!(name, addr, num_threads, WatchdogMakeSvc { _runner: runner });
        }

        WatchdogMode::ModeSerializing => {
            let runner = SerializingForkRunner::new(config)?;
            build_and_serve!(name, addr, num_threads, WatchdogMakeSvc { _runner: runner });
        }

        WatchdogMode::ModeWasm => {
            #[cfg(feature = "wasm")]
            {
                let runner = WasmRunner::new(config)?;
                build_and_serve!(name, addr, num_threads, WatchdogMakeSvc { _runner: runner });
            }
            #[cfg(not(feature = "wasm"))]
            return Err(anyhow!("`wasm` feature doest not be enable"));
        }

        _ => {
            return Err(anyhow!(
                "watchdog mode {} is not yet implemented",
                config._operational_mode
            ));
        }
    }

    Ok(())
}
