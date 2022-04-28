use std::sync::Arc;
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use super::Handler;
use crate::WatchdogConfig;


struct MetricsHandler {}

impl Handler for MetricsHandler {
    fn handle(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
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
}


pub(super) fn make_handler(_config: WatchdogConfig) -> Result<Arc<dyn Handler + Send + Sync>> {
    Ok(Arc::new(MetricsHandler {}))
}
