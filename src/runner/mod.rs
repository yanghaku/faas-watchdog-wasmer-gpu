/// for wasm mode
#[cfg(feature = "wasm")]
pub(crate) mod wasm_runner;

/// for stream mode
mod forking_runner;

/// for http mode
mod http_runner;

/// for static mode
mod static_file_processor;

/// for serial mode
mod serializing_fork_runner;

use anyhow::Result;
use hyper::body::Bytes;
use hyper::http::{request, response};
use hyper::Body;
use tokio::sync::{mpsc, oneshot};

/// parse the request and run function and generate the response
pub(crate) trait Runner {
    /// run function request
    fn run(
        &self,
        _req_head: request::Parts,
        _req_body: mpsc::Receiver<Result<Bytes, hyper::Error>>,
        _res_head: &mut response::Parts,
    ) -> oneshot::Receiver<Result<Body>> {
        todo!()
    }

    /// get the scale number tuple: (now replicas, available replicas, invoke count)
    fn get_scale(&self) -> (usize, usize, usize) {
        // default is return zero
        (0, 0, 0)
    }

    /// update replicas
    fn set_scale(&self, _replicas: usize) -> Result<()> {
        // default is do nothing
        Ok(())
    }
}

pub(crate) use forking_runner::*;
pub(crate) use http_runner::*;
pub(crate) use serializing_fork_runner::*;
pub(crate) use static_file_processor::*;
#[cfg(feature = "wasm")]
pub(crate) use wasm_runner::*;
