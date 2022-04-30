/// some help function
mod utils;

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
use hyper::{Body, Request, Response};


/// parse the request and run function and generate the response
pub(crate) trait Runner {
    fn run(&self, _request: Request<Body>, _: &mut Response<Body>) -> Result<()>;

    fn scale(&self, _replicas: usize) -> Result<()> {
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
