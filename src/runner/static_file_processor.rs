use crate::runner::Runner;
use crate::WatchdogConfig;
use anyhow::Result;
use hyper::{Body, Request, Response};

#[derive(Clone)]
pub(crate) struct StaticFileProcessor;

impl Runner for StaticFileProcessor {
    fn run(&self, _: Request<Body>, _: &mut Response<Body>) -> Result<()> {
        todo!()
    }
}

impl StaticFileProcessor {
    pub(crate) fn new(config: WatchdogConfig) -> Result<Self> {
        eprintln!("{:?}", config);
        todo!()
    }
}
