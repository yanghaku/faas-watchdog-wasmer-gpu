use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use crate::server::Handler;
use crate::WatchdogConfig;


struct MetricsHandler {}

impl Handler for MetricsHandler {
    fn handle(&mut self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let mut response = Response::default(); // default is 200 OK
        match req.uri().path() {
            "/metrics" => {
                todo!()
            }
            _ => {
                *response.status_mut() = StatusCode::NOT_FOUND;
            }
        }
        Ok(response)
    }
}


pub(crate) fn make_handler(_config: WatchdogConfig) -> Result<impl Handler + Send> {
    Ok(MetricsHandler {})
}
