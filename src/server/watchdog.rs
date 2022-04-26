use anyhow::Result;
use hyper::{Body, Method, Request, Response, StatusCode};
use crate::server::Handler;
use crate::WatchdogConfig;
use crate::health::check_healthy;

struct WatchdogHandler {}


impl Handler for WatchdogHandler {
    fn handle(&mut self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let mut response = Response::default(); // default is 200 OK

        match req.method() {
            &Method::GET => {
                match req.uri().path() {
                    "/" => {
                        todo!()
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


pub(crate) fn make_handler(_config: WatchdogConfig) -> Result<impl Handler + Send> {
    return Ok(WatchdogHandler {});
}
