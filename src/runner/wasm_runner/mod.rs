use anyhow::Result;
use hyper::{Body, Request, Response};
use crate::runner::{Runner, WasmRunner};
use crate::WatchdogConfig;

/// compile the wasm module to native dylib
mod compiler;

pub(crate) struct Compiler {}


impl Runner for WasmRunner {
    fn run(&self, req: &mut Request<Body>, res: &mut Response<Body>) -> Result<()> {
        *res.body_mut() = Body::from(format!("{:?}", req));

        Ok(())
    }
}


impl WasmRunner {
    pub(crate) fn new(config: WatchdogConfig) -> Result<Self> {
        eprintln!("{:?}", config);
        Ok(Self {})
    }
}
