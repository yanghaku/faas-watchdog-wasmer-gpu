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
    fn run(&self, _: &mut Request<Body>, _: &mut Response<Body>) -> Result<()>;
}


#[cfg(feature = "wasm")]
pub(crate) struct WasmRunner {
    /// compiled wasm module
    _module: wasmer::Module,

    /// the function process and arguments
    _func_process: Vec<String>,

    /// workplace root directory
    _wasm_root: std::path::PathBuf,
}


pub(crate) struct ForkingRunner;


pub(crate) struct HttpRunner;


pub(crate) struct StaticFileProcessor;


pub(crate) struct SerializingForkRunner;
