use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use hyper::service::Service;

use crate::WatchdogConfig;
use super::shutdown_signal;


pub(super) struct MetricsMakeSvc;

impl<T> Service<T> for MetricsMakeSvc {
    type Response = MetricsService;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let fut = async move { Ok(MetricsService {}) };
        Box::pin(fut)
    }
}


pub(super) struct MetricsService;


impl Service<Request<Body>> for MetricsService {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::pin(handle(req))
    }
}


async fn handle(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut response = Response::default(); // default is 200 OK
    match req.uri().path() {
        "/metrics" => {
            // todo: impl it
            *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    }
    Ok(response)
}


/// build watchdog server and serve
pub(super) fn build_and_serve(name: &'static str, addr: SocketAddr,
                              num_threads: usize, _config: WatchdogConfig) -> Result<()> {
    build_and_serve!(name,addr,num_threads,MetricsMakeSvc{});
    Ok(())
}
