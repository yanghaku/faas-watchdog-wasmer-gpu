use std::net::SocketAddr;

use anyhow::Result;
use hyper::header::CONTENT_TYPE;
use hyper::http::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, StatusCode};
use lazy_static::lazy_static;
use prometheus::{register_counter_vec, register_gauge, register_histogram_vec};
use prometheus::{CounterVec, Encoder, Gauge, HistogramVec, TextEncoder};

use super::shutdown_signal;

// global variables, register the metrics
lazy_static! {
    /// text encoder for metrics result
    static ref ENCODER: TextEncoder = TextEncoder::new();
    /// content type value
    static ref CONTENT_TYPE_VALUE: HeaderValue = ENCODER.format_type().parse().unwrap();
    /// in flight: the number of functions which are running
    pub(super) static ref IN_FLIGHT: Gauge =
        register_gauge!("requests_in_flight", "total HTTP requests in-flight").unwrap();
    /// the request count
    pub(super) static ref REQUESTS_TOTAL: CounterVec = register_counter_vec!(
        "requests_total",
        "total HTTP requests processed",
        &["code", "method"],
    )
    .unwrap();
    /// the running time
    pub(super) static ref REQUEST_DURATION_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "request_duration_seconds",
        "Seconds spent serving HTTP requests.",
        &["code", "method"],
    )
    .unwrap();
}

async fn handle(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut response = Response::default(); // default is 200 OK
    match req.uri().path() {
        "/metrics" => {
            let metric_families = prometheus::gather();
            let mut buffer = vec![];
            if let Err(e) = ENCODER.encode(&metric_families, &mut buffer) {
                *response.body_mut() = Body::from(format!("Encode error: {:?}", e));
            } else {
                response
                    .headers_mut()
                    .insert(CONTENT_TYPE, CONTENT_TYPE_VALUE.clone());
                *response.body_mut() = Body::from(buffer);
            }
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    }
    Ok(response)
}

/// build watchdog server and serve
pub(super) fn build_and_serve(
    name: &'static str,
    addr: SocketAddr,
    num_threads: usize,
) -> Result<()> {
    // init the metrics value
    IN_FLIGHT.set(0 as f64);

    build_and_serve!(
        name,
        addr,
        num_threads,
        make_service_fn(|_| { async { Ok::<_, hyper::Error>(service_fn(|req: _| handle(req))) } })
    );
    Ok(())
}
